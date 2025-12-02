//! GitHub Scraper - Enriches implementation records with GitHub repository statistics
//!
//! This scraper fetches GitHub API data for repositories linked to papers
//! and updates the implementations table with stars, forks, and other metadata.

use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use serde::Deserialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

const USER_AGENT: &str = "CodeWithPapers-Replicator/1.0 (Educational/Research Purpose; https://github.com/GeorgePearse/codewithpapers)";

#[derive(Parser, Debug)]
#[command(author, version, about = "Scrape GitHub stats for paper implementations", long_about = None)]
struct Args {
    /// Maximum number of repos to process (0 = all)
    #[arg(short, long, default_value_t = 0)]
    max_repos: usize,

    /// Delay between requests in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    delay_ms: u64,

    /// GitHub API token (can also use GITHUB_TOKEN env var)
    #[arg(long)]
    token: Option<String>,

    /// Only process repos that haven't been updated recently
    #[arg(long, default_value_t = false)]
    stale_only: bool,

    /// Dry run - don't write to database
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    stargazers_count: i32,
    forks_count: i32,
    open_issues_count: i32,
    subscribers_count: Option<i32>,
    language: Option<String>,
    description: Option<String>,
    archived: bool,
    disabled: bool,
    pushed_at: Option<String>,
    topics: Option<Vec<String>>,
}

#[derive(Debug)]
struct Implementation {
    id: uuid::Uuid,
    github_url: String,
}

#[derive(Debug, Default)]
struct ScraperStats {
    repos_found: usize,
    repos_processed: usize,
    repos_updated: usize,
    repos_not_found: usize,
    rate_limited: usize,
    errors: usize,
}

struct GitHubScraper {
    client: reqwest::Client,
    pool: Option<PgPool>,
    delay: Duration,
    dry_run: bool,
    stats: ScraperStats,
}

impl GitHubScraper {
    async fn new(
        pool: Option<PgPool>,
        delay_ms: u64,
        dry_run: bool,
        token: Option<String>,
    ) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse()?,
        );

        if let Some(ref token) = token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse()?,
            );
            info!("Using GitHub API token for authentication");
        } else {
            warn!("No GitHub token provided - rate limits will be very restrictive (60 req/hour)");
        }

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            pool,
            delay: Duration::from_millis(delay_ms),
            dry_run,
            stats: ScraperStats::default(),
        })
    }

    fn parse_github_url(&self, url: &str) -> Option<(String, String)> {
        // Parse GitHub URL to extract owner and repo
        // Handles: https://github.com/owner/repo, https://github.com/owner/repo.git, etc.
        let url = url.trim_end_matches(".git").trim_end_matches('/');

        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            let repo = parts.last()?.to_string();
            let owner = parts.get(parts.len() - 2)?.to_string();

            if !owner.is_empty() && !repo.is_empty() && owner != "github.com" {
                return Some((owner, repo));
            }
        }
        None
    }

    async fn fetch_repo_stats(&self, owner: &str, repo: &str) -> Result<Option<GitHubRepo>> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        debug!("Fetching: {}", url);

        sleep(self.delay).await;

        let resp = self.client.get(&url).send().await?;

        let status = resp.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            debug!("Repository not found: {}/{}", owner, repo);
            return Ok(None);
        }

        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            warn!("Rate limited by GitHub API");
            return Err(anyhow::anyhow!("Rate limited"));
        }

        if !status.is_success() {
            return Err(anyhow::anyhow!("HTTP {} for {}", status, url));
        }

        let repo_data: GitHubRepo = resp.json().await?;
        Ok(Some(repo_data))
    }

    async fn get_implementations(&self, pool: &PgPool, limit: usize) -> Result<Vec<Implementation>> {
        let query = if limit > 0 {
            sqlx::query(
                r#"
                SELECT id, github_url
                FROM implementations
                WHERE github_url IS NOT NULL AND github_url != ''
                ORDER BY updated_at ASC NULLS FIRST
                LIMIT $1
                "#
            )
            .bind(limit as i64)
        } else {
            sqlx::query(
                r#"
                SELECT id, github_url
                FROM implementations
                WHERE github_url IS NOT NULL AND github_url != ''
                ORDER BY updated_at ASC NULLS FIRST
                "#
            )
        };

        let rows = query.fetch_all(pool).await?;

        let implementations: Vec<Implementation> = rows
            .iter()
            .map(|row| Implementation {
                id: row.get("id"),
                github_url: row.get("github_url"),
            })
            .collect();

        Ok(implementations)
    }

    async fn update_implementation(
        &self,
        pool: &PgPool,
        impl_id: uuid::Uuid,
        repo: &GitHubRepo,
        framework: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE implementations
            SET stars = $1,
                framework = COALESCE($2, framework),
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(repo.stargazers_count)
        .bind(framework)
        .bind(impl_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn run(&mut self, max_repos: usize) -> Result<()> {
        let pool = match &self.pool {
            Some(p) => p,
            None => {
                info!("Dry run mode - no database operations");
                return Ok(());
            }
        };

        let implementations = self.get_implementations(pool, max_repos).await?;
        self.stats.repos_found = implementations.len();
        info!("Found {} implementations to process", implementations.len());

        for imp in &implementations {
            if let Some((owner, repo)) = self.parse_github_url(&imp.github_url) {
                match self.fetch_repo_stats(&owner, &repo).await {
                    Ok(Some(repo_data)) => {
                        let framework = repo_data.language.as_deref();

                        if !self.dry_run {
                            if let Some(pool) = &self.pool {
                                match self.update_implementation(pool, imp.id, &repo_data, framework).await {
                                    Ok(_) => {
                                        debug!(
                                            "Updated {}/{}: {} stars",
                                            owner, repo, repo_data.stargazers_count
                                        );
                                        self.stats.repos_updated += 1;
                                    }
                                    Err(e) => {
                                        warn!("Failed to update implementation: {}", e);
                                        self.stats.errors += 1;
                                    }
                                }
                            }
                        } else {
                            debug!(
                                "[DRY RUN] Would update {}/{}: {} stars, lang: {:?}",
                                owner, repo, repo_data.stargazers_count, framework
                            );
                            self.stats.repos_updated += 1;
                        }
                        self.stats.repos_processed += 1;
                    }
                    Ok(None) => {
                        self.stats.repos_not_found += 1;
                        self.stats.repos_processed += 1;
                    }
                    Err(e) => {
                        if e.to_string().contains("Rate limited") {
                            self.stats.rate_limited += 1;
                            warn!("Rate limited - stopping scraper");
                            break;
                        }
                        error!("Error fetching {}/{}: {}", owner, repo, e);
                        self.stats.errors += 1;
                    }
                }
            } else {
                debug!("Could not parse GitHub URL: {}", imp.github_url);
                self.stats.errors += 1;
            }
        }

        Ok(())
    }

    fn print_stats(&self) {
        info!("=== GitHub Scraper Statistics ===");
        info!("Repos found: {}", self.stats.repos_found);
        info!("Repos processed: {}", self.stats.repos_processed);
        info!("Repos updated: {}", self.stats.repos_updated);
        info!("Repos not found (404): {}", self.stats.repos_not_found);
        info!("Rate limited: {}", self.stats.rate_limited);
        info!("Errors: {}", self.stats.errors);
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

    info!("Starting GitHub Scraper...");
    if args.dry_run {
        warn!("DRY RUN MODE - No database writes will occur");
    }

    // Get GitHub token
    let token = args.token.or_else(|| env::var("GITHUB_TOKEN").ok());

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

    let mut scraper = GitHubScraper::new(pool, args.delay_ms, args.dry_run, token).await?;
    scraper.run(args.max_repos).await?;
    scraper.print_stats();

    info!("GitHub scraping complete.");
    Ok(())
}
