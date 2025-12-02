//! Build Tantivy Search Index
//!
//! Indexes all papers from PostgreSQL into the Tantivy full-text search index.
//!
//! Usage:
//!     build_search_index
//!     build_search_index --index-path ./data/tantivy_index

use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use backend::search::SearchIndex;
use backend::Paper;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Build Tantivy search index from PostgreSQL papers",
    long_about = "Indexes all papers from the database into a Tantivy full-text search index.\n\
                  This should be run once initially and after bulk data loads."
)]
struct Args {
    /// Path for the Tantivy index
    #[arg(long, default_value = "./data/tantivy_index")]
    index_path: PathBuf,

    /// Batch size for fetching papers
    #[arg(long, default_value_t = 10000)]
    batch_size: i64,

    /// Commit interval (number of documents between commits)
    #[arg(long, default_value_t = 50000)]
    commit_interval: usize,

    /// Force rebuild (delete existing index)
    #[arg(long, default_value_t = false)]
    force: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
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

    // Force rebuild if requested
    if args.force && args.index_path.exists() {
        info!("Removing existing index at {:?}", args.index_path);
        std::fs::remove_dir_all(&args.index_path)?;
    }

    // Connect to database
    let database_url = env::var("POSTGRES_URI")
        .or_else(|_| env::var("DATABASE_URL"))
        .context("POSTGRES_URI or DATABASE_URL must be set")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    info!("Connected to database");

    // Get total paper count
    let (total_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM papers")
        .fetch_one(&pool)
        .await
        .context("Failed to get paper count")?;

    info!("Total papers to index: {}", total_count);

    // Create or open index
    let search_index = SearchIndex::open_or_create(&args.index_path)
        .context("Failed to create/open search index")?;

    info!("Index ready at {:?}", args.index_path);

    // Create writer with 50MB heap
    let mut writer = search_index.writer(50_000_000)?;

    let mut indexed_count = 0usize;
    let mut offset = 0i64;

    loop {
        // Fetch batch of papers
        let papers: Vec<Paper> = sqlx::query_as(
            r#"
            SELECT id, title, abstract, arxiv_id, arxiv_url, pdf_url,
                   published_date, authors, created_at, updated_at
            FROM papers
            ORDER BY id
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(args.batch_size)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .context("Failed to fetch papers")?;

        if papers.is_empty() {
            break;
        }

        let batch_size = papers.len();

        // Index each paper
        for paper in &papers {
            let doc = search_index.paper_to_document(paper);
            writer.add_document(doc)?;
            indexed_count += 1;

            // Commit periodically
            if indexed_count % args.commit_interval == 0 {
                info!(
                    "Committing at {} documents ({:.1}%)",
                    indexed_count,
                    (indexed_count as f64 / total_count as f64) * 100.0
                );
                writer.commit()?;
            }
        }

        info!(
            "Indexed batch of {} papers (total: {}/{}, {:.1}%)",
            batch_size,
            indexed_count,
            total_count,
            (indexed_count as f64 / total_count as f64) * 100.0
        );

        offset += args.batch_size;
    }

    // Final commit
    info!("Final commit...");
    writer.commit()?;

    info!(
        "Indexing complete! {} papers indexed to {:?}",
        indexed_count, args.index_path
    );

    Ok(())
}
