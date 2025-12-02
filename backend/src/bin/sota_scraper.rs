//! SOTA Scraper - Scrapes Papers with Code state-of-the-art leaderboards from Wayback Machine
//!
//! This scraper fetches archived snapshots of paperswithcode.com SOTA pages
//! and populates the database with tasks, datasets, and benchmarks.

use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use scraper::{Html, Selector};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::collections::HashSet;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

const DEFAULT_SOTA_URL: &str =
    "https://web.archive.org/web/20250117073537/https://paperswithcode.com/sota";
const USER_AGENT: &str = "CodeWithPapers-Replicator/1.0 (Educational/Research Purpose; https://github.com/GeorgePearse/codewithpapers)";

#[derive(Parser, Debug)]
#[command(author, version, about = "Scrape Papers with Code SOTA leaderboards", long_about = None)]
struct Args {
    /// Maximum number of tasks to scrape (0 = all)
    #[arg(short, long, default_value_t = 0)]
    max_tasks: usize,

    /// Delay between requests in milliseconds
    #[arg(short, long, default_value_t = 2000)]
    delay_ms: u64,

    /// Custom SOTA page URL (Wayback Machine archive)
    #[arg(long)]
    sota_url: Option<String>,

    /// Dry run - don't write to database
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, Clone)]
struct Task {
    name: String,
    url: String,
}

#[derive(Debug)]
struct ScraperStats {
    tasks_found: usize,
    tasks_processed: usize,
    datasets_inserted: usize,
    benchmarks_inserted: usize,
    errors: usize,
}

impl Default for ScraperStats {
    fn default() -> Self {
        Self {
            tasks_found: 0,
            tasks_processed: 0,
            datasets_inserted: 0,
            benchmarks_inserted: 0,
            errors: 0,
        }
    }
}

struct Scraper {
    client: reqwest::Client,
    pool: Option<PgPool>,
    delay: Duration,
    dry_run: bool,
    stats: ScraperStats,
    seen_datasets: HashSet<String>,
}

impl Scraper {
    async fn new(pool: Option<PgPool>, delay_ms: u64, dry_run: bool) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            pool,
            delay: Duration::from_millis(delay_ms),
            dry_run,
            stats: ScraperStats::default(),
            seen_datasets: HashSet::new(),
        })
    }

    async fn fetch_page(&self, url: &str) -> Result<String> {
        debug!("Fetching: {}", url);
        sleep(self.delay).await;

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("HTTP {} for {}", status, url);
        }

        let body = resp.text().await.context("Failed to read response body")?;
        Ok(body)
    }

    async fn scrape_sota_page(&mut self, url: &str) -> Result<Vec<Task>> {
        info!("Scraping SOTA page: {}", url);
        let html = self.fetch_page(url).await?;
        let document = Html::parse_document(&html);

        let card_selector =
            Selector::parse(".card").expect("Invalid selector");
        let title_selector =
            Selector::parse(".card-col-title h1").expect("Invalid selector");
        let link_selector = Selector::parse("a").expect("Invalid selector");

        let mut tasks = Vec::new();

        for card in document.select(&card_selector) {
            if let Some(title_elem) = card.select(&title_selector).next() {
                let title = title_elem.text().collect::<String>().trim().to_string();

                if title.is_empty() {
                    continue;
                }

                if let Some(link_elem) = card.select(&link_selector).next() {
                    if let Some(href) = link_elem.value().attr("href") {
                        let full_url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("https://web.archive.org{}", href)
                        };

                        tasks.push(Task {
                            name: title,
                            url: full_url,
                        });
                    }
                }
            }
        }

        self.stats.tasks_found = tasks.len();
        info!("Found {} tasks on SOTA page", tasks.len());
        Ok(tasks)
    }

    async fn scrape_task_details(&mut self, task: &Task) -> Result<usize> {
        info!("Scraping task: {}", task.name);
        let html = self.fetch_page(&task.url).await?;
        let document = Html::parse_document(&html);

        let link_selector = Selector::parse("a").expect("Invalid selector");

        let mut datasets_found = 0;

        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                if href.contains("/dataset/") {
                    let text = link.text().collect::<String>().trim().to_string();
                    if !text.is_empty() && !self.seen_datasets.contains(&text) {
                        self.seen_datasets.insert(text.clone());

                        if !self.dry_run {
                            if let Some(pool) = &self.pool {
                                match self.insert_dataset_and_benchmark(pool, &text, &task.name).await {
                                    Ok(_) => {
                                        datasets_found += 1;
                                        self.stats.datasets_inserted += 1;
                                        self.stats.benchmarks_inserted += 1;
                                    }
                                    Err(e) => {
                                        warn!("Failed to insert dataset '{}': {}", text, e);
                                        self.stats.errors += 1;
                                    }
                                }
                            }
                        } else {
                            debug!("  [DRY RUN] Would insert dataset: {}", text);
                            datasets_found += 1;
                        }
                    }
                }
            }
        }

        debug!(
            "  -> Found {} new datasets for task '{}'",
            datasets_found, task.name
        );
        Ok(datasets_found)
    }

    async fn insert_dataset_and_benchmark(
        &self,
        pool: &PgPool,
        dataset_name: &str,
        task_name: &str,
    ) -> Result<()> {
        // Insert dataset
        let row = sqlx::query(
            r#"
            INSERT INTO datasets (name, description)
            VALUES ($1, 'Imported from SOTA scrape')
            ON CONFLICT (name) DO UPDATE SET updated_at = NOW()
            RETURNING id
            "#,
        )
        .bind(dataset_name)
        .fetch_one(pool)
        .await
        .context("Failed to insert dataset")?;

        let dataset_id: uuid::Uuid = row.try_get("id")?;

        // Insert benchmark
        let benchmark_name = format!("{} on {}", task_name, dataset_name);

        sqlx::query(
            r#"
            INSERT INTO benchmarks (name, dataset_id, task, description)
            VALUES ($1, $2, $3, 'Imported from SOTA scrape')
            ON CONFLICT (name, dataset_id) DO NOTHING
            "#,
        )
        .bind(&benchmark_name)
        .bind(dataset_id)
        .bind(task_name)
        .execute(pool)
        .await
        .context("Failed to insert benchmark")?;

        debug!("Inserted: {} -> {}", dataset_name, benchmark_name);
        Ok(())
    }

    fn print_stats(&self) {
        info!("=== Scraper Statistics ===");
        info!("Tasks found: {}", self.stats.tasks_found);
        info!("Tasks processed: {}", self.stats.tasks_processed);
        info!("Datasets inserted: {}", self.stats.datasets_inserted);
        info!("Benchmarks inserted: {}", self.stats.benchmarks_inserted);
        info!("Errors: {}", self.stats.errors);
        info!(
            "Unique datasets seen: {}",
            self.seen_datasets.len()
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let args = Args::parse();

    // Setup logging
    let log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting SOTA Scraper...");
    if args.dry_run {
        warn!("DRY RUN MODE - No database writes will occur");
    }

    // Connect to database (unless dry run)
    let pool = if args.dry_run {
        None
    } else {
        let database_url = env::var("POSTGRES_URI").context("POSTGRES_URI must be set")?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .context("Failed to connect to database")?;
        info!("Connected to database");
        Some(pool)
    };

    let mut scraper = Scraper::new(pool, args.delay_ms, args.dry_run).await?;

    // Scrape SOTA page
    let sota_url = args.sota_url.as_deref().unwrap_or(DEFAULT_SOTA_URL);
    let tasks = scraper.scrape_sota_page(sota_url).await?;

    // Determine how many tasks to process
    let tasks_to_process: Vec<_> = if args.max_tasks > 0 {
        tasks.into_iter().take(args.max_tasks).collect()
    } else {
        tasks
    };

    info!("Processing {} tasks...", tasks_to_process.len());

    // Process each task
    for task in &tasks_to_process {
        match scraper.scrape_task_details(task).await {
            Ok(_) => {
                scraper.stats.tasks_processed += 1;
            }
            Err(e) => {
                error!("Error scraping task '{}': {}", task.name, e);
                scraper.stats.errors += 1;
            }
        }
    }

    scraper.print_stats();
    info!("Scraping complete.");

    Ok(())
}
