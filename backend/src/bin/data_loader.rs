//! Data Loader - Load Papers with Code archive data from parquet files to PostgreSQL
//!
//! This is a high-performance Rust replacement for the Python data loading scripts.
//! It reads parquet files and streams data directly to the database.

use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::RowAccessor;
use serde_json::json;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about = "Load PWC archive data into PostgreSQL", long_about = None)]
struct Args {
    /// Data directory containing parquet files
    #[arg(short, long, default_value = "data/pwc-archive")]
    data_dir: PathBuf,

    /// Batch size for database inserts
    #[arg(short, long, default_value_t = 1000)]
    batch_size: usize,

    /// Only load specific dataset (papers, datasets, links, methods)
    #[arg(long)]
    only: Option<String>,

    /// Skip papers that are already loaded
    #[arg(long, default_value_t = true)]
    skip_existing: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

struct LoaderStats {
    papers_inserted: AtomicUsize,
    papers_skipped: AtomicUsize,
    datasets_inserted: AtomicUsize,
    links_inserted: AtomicUsize,
    errors: AtomicUsize,
}

impl Default for LoaderStats {
    fn default() -> Self {
        Self {
            papers_inserted: AtomicUsize::new(0),
            papers_skipped: AtomicUsize::new(0),
            datasets_inserted: AtomicUsize::new(0),
            links_inserted: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
        }
    }
}

async fn load_papers(pool: &PgPool, data_dir: &PathBuf, stats: &LoaderStats) -> Result<()> {
    let parquet_path = data_dir.join("papers-with-abstracts/train.parquet");

    if !parquet_path.exists() {
        warn!("Papers parquet file not found: {:?}", parquet_path);
        return Ok(());
    }

    info!("Loading papers from {:?}", parquet_path);

    let file = File::open(&parquet_path)?;
    let reader = SerializedFileReader::new(file)?;
    let mut iter = reader.get_row_iter(None)?;

    let mut batch_count = 0;
    let total_rows = reader.metadata().file_metadata().num_rows() as usize;

    info!("Total papers in file: {}", total_rows);

    while let Some(row_result) = iter.next() {
        let row = row_result?;

        // Extract fields
        let arxiv_id = row.get_string(1).ok().map(|s| s.to_string());
        let title = row.get_string(4).ok().map(|s| s.to_string());
        let abstract_text = row.get_string(5).ok().map(|s| s.to_string());
        let url_abs = row.get_string(7).ok().map(|s| s.to_string());
        let url_pdf = row.get_string(8).ok().map(|s| s.to_string());

        // Skip papers without arxiv_id
        let arxiv_id = match arxiv_id {
            Some(id) if !id.is_empty() => id,
            _ => {
                stats.papers_skipped.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        // Insert paper
        let result = sqlx::query(
            r#"
            INSERT INTO papers (title, abstract, arxiv_id, arxiv_url, pdf_url)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (arxiv_id) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(&title)
        .bind(&abstract_text)
        .bind(&arxiv_id)
        .bind(&url_abs)
        .bind(&url_pdf)
        .fetch_optional(pool)
        .await;

        match result {
            Ok(Some(_)) => {
                stats.papers_inserted.fetch_add(1, Ordering::Relaxed);
            }
            Ok(None) => {
                stats.papers_skipped.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                debug!("Error inserting paper {}: {}", arxiv_id, e);
                stats.errors.fetch_add(1, Ordering::Relaxed);
            }
        }

        batch_count += 1;
        if batch_count % 10000 == 0 {
            let inserted = stats.papers_inserted.load(Ordering::Relaxed);
            let skipped = stats.papers_skipped.load(Ordering::Relaxed);
            info!(
                "Progress: {}/{} papers ({} inserted, {} skipped)",
                batch_count, total_rows, inserted, skipped
            );
        }
    }

    Ok(())
}

async fn load_datasets(pool: &PgPool, data_dir: &PathBuf, stats: &LoaderStats) -> Result<()> {
    let parquet_path = data_dir.join("datasets/train.parquet");

    if !parquet_path.exists() {
        warn!("Datasets parquet file not found: {:?}", parquet_path);
        return Ok(());
    }

    info!("Loading datasets from {:?}", parquet_path);

    let file = File::open(&parquet_path)?;
    let reader = SerializedFileReader::new(file)?;
    let mut iter = reader.get_row_iter(None)?;

    while let Some(row_result) = iter.next() {
        let row = row_result?;

        let name = row.get_string(0).ok().map(|s| s.to_string());
        let description = row.get_string(2).ok().map(|s| s.to_string());
        let homepage = row.get_string(4).ok().map(|s| s.to_string());

        let name = match name {
            Some(n) if !n.is_empty() => n,
            _ => continue,
        };

        let result = sqlx::query(
            r#"
            INSERT INTO datasets (name, description, homepage_url)
            VALUES ($1, $2, $3)
            ON CONFLICT (name) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(&homepage)
        .fetch_optional(pool)
        .await;

        match result {
            Ok(Some(_)) => {
                stats.datasets_inserted.fetch_add(1, Ordering::Relaxed);
            }
            Ok(None) => {}
            Err(e) => {
                debug!("Error inserting dataset {}: {}", name, e);
                stats.errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    info!(
        "Datasets loaded: {} inserted",
        stats.datasets_inserted.load(Ordering::Relaxed)
    );
    Ok(())
}

async fn load_links(pool: &PgPool, data_dir: &PathBuf, stats: &LoaderStats) -> Result<()> {
    let parquet_path = data_dir.join("links-between-paper-and-code/train.parquet");

    if !parquet_path.exists() {
        warn!("Links parquet file not found: {:?}", parquet_path);
        return Ok(());
    }

    info!("Loading code links from {:?}", parquet_path);

    let file = File::open(&parquet_path)?;
    let reader = SerializedFileReader::new(file)?;
    let mut iter = reader.get_row_iter(None)?;

    let total_rows = reader.metadata().file_metadata().num_rows() as usize;
    let mut processed = 0;

    while let Some(row_result) = iter.next() {
        let row = row_result?;

        // Get arxiv_id and repo_url
        let arxiv_id = row.get_string(1).ok().map(|s| s.to_string());
        let repo_url = row.get_string(3).ok().map(|s| s.to_string());
        let framework = row.get_string(5).ok().map(|s| s.to_string());

        let arxiv_id = match arxiv_id {
            Some(id) if !id.is_empty() => id,
            _ => continue,
        };

        let repo_url = match repo_url {
            Some(url) if !url.is_empty() => url,
            _ => continue,
        };

        // First get the paper_id
        let paper_result: Option<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM papers WHERE arxiv_id = $1 LIMIT 1"
        )
        .bind(&arxiv_id)
        .fetch_optional(pool)
        .await?;

        let paper_id = match paper_result {
            Some((id,)) => id,
            None => continue, // Paper not in database
        };

        // Insert implementation
        let result = sqlx::query(
            r#"
            INSERT INTO implementations (paper_id, github_url, framework)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            RETURNING id
            "#,
        )
        .bind(paper_id)
        .bind(&repo_url)
        .bind(&framework)
        .fetch_optional(pool)
        .await;

        match result {
            Ok(Some(_)) => {
                stats.links_inserted.fetch_add(1, Ordering::Relaxed);
            }
            Ok(None) => {}
            Err(e) => {
                debug!("Error inserting link: {}", e);
                stats.errors.fetch_add(1, Ordering::Relaxed);
            }
        }

        processed += 1;
        if processed % 50000 == 0 {
            info!(
                "Progress: {}/{} links ({} inserted)",
                processed,
                total_rows,
                stats.links_inserted.load(Ordering::Relaxed)
            );
        }
    }

    info!(
        "Links loaded: {} inserted",
        stats.links_inserted.load(Ordering::Relaxed)
    );
    Ok(())
}

fn print_stats(stats: &LoaderStats) {
    info!("=== Loading Statistics ===");
    info!(
        "Papers: {} inserted, {} skipped",
        stats.papers_inserted.load(Ordering::Relaxed),
        stats.papers_skipped.load(Ordering::Relaxed)
    );
    info!(
        "Datasets: {} inserted",
        stats.datasets_inserted.load(Ordering::Relaxed)
    );
    info!(
        "Links: {} inserted",
        stats.links_inserted.load(Ordering::Relaxed)
    );
    info!("Errors: {}", stats.errors.load(Ordering::Relaxed));
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
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Rust Data Loader...");
    info!("Data directory: {:?}", args.data_dir);

    // Connect to database
    let database_url = env::var("POSTGRES_URI").context("POSTGRES_URI must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;
    info!("Connected to database");

    let stats = LoaderStats::default();

    // Load data based on --only flag or all
    match args.only.as_deref() {
        Some("papers") => {
            load_papers(&pool, &args.data_dir, &stats).await?;
        }
        Some("datasets") => {
            load_datasets(&pool, &args.data_dir, &stats).await?;
        }
        Some("links") => {
            load_links(&pool, &args.data_dir, &stats).await?;
        }
        Some(other) => {
            error!("Unknown dataset: {}. Use: papers, datasets, links", other);
        }
        None => {
            // Load all in order
            load_papers(&pool, &args.data_dir, &stats).await?;
            load_datasets(&pool, &args.data_dir, &stats).await?;
            load_links(&pool, &args.data_dir, &stats).await?;
        }
    }

    print_stats(&stats);
    info!("Loading complete.");

    Ok(())
}
