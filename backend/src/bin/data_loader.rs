//! Data Loader - Load Papers with Code archive data from parquet files to PostgreSQL
//!
//! High-performance batch loader using Arrow columnar API and UNNEST for bulk inserts.

use anyhow::{Context, Result};
use arrow::array::{Array, StringArray};
use arrow::record_batch::RecordBatch;
use clap::Parser;
use dotenvy::dotenv;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about = "Load PWC archive data into PostgreSQL", long_about = None)]
struct Args {
    /// Data directory containing parquet files
    #[arg(short, long, default_value = "data/pwc-archive")]
    data_dir: PathBuf,

    /// Batch size for database inserts (smaller = more reliable for serverless)
    #[arg(short, long, default_value_t = 500)]
    batch_size: usize,

    /// Only load specific dataset (papers, datasets, links)
    #[arg(long)]
    only: Option<String>,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Default)]
struct LoaderStats {
    papers_inserted: usize,
    papers_skipped: usize,
    datasets_inserted: usize,
    links_inserted: usize,
}

async fn insert_paper_batch(
    pool: &PgPool,
    titles: &[Option<String>],
    abstracts: &[Option<String>],
    arxiv_ids: &[String],
    arxiv_urls: &[Option<String>],
    pdf_urls: &[Option<String>],
) -> Result<usize> {
    if arxiv_ids.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO papers (title, abstract, arxiv_id, arxiv_url, pdf_url)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::text[])
        ON CONFLICT (arxiv_id) DO NOTHING
        "#,
    )
    .bind(titles)
    .bind(abstracts)
    .bind(arxiv_ids)
    .bind(arxiv_urls)
    .bind(pdf_urls)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as usize)
}

async fn insert_dataset_batch(
    pool: &PgPool,
    names: &[String],
    descriptions: &[Option<String>],
    homepage_urls: &[Option<String>],
) -> Result<usize> {
    if names.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO datasets (name, description, homepage_url)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])
        ON CONFLICT (name) DO NOTHING
        "#,
    )
    .bind(names)
    .bind(descriptions)
    .bind(homepage_urls)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as usize)
}

async fn insert_link_batch(
    pool: &PgPool,
    arxiv_ids: &[String],
    repo_urls: &[String],
    frameworks: &[Option<String>],
) -> Result<usize> {
    if arxiv_ids.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO implementations (paper_id, github_url, framework)
        SELECT p.id, links.repo_url, links.framework
        FROM UNNEST($1::text[], $2::text[], $3::text[]) AS links(arxiv_id, repo_url, framework)
        JOIN papers p ON p.arxiv_id = links.arxiv_id
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(arxiv_ids)
    .bind(repo_urls)
    .bind(frameworks)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as usize)
}

fn get_string_column(batch: &RecordBatch, col_idx: usize) -> Option<&StringArray> {
    batch
        .column(col_idx)
        .as_any()
        .downcast_ref::<StringArray>()
}

async fn load_papers(
    pool: &PgPool,
    data_dir: &PathBuf,
    batch_size: usize,
    stats: &mut LoaderStats,
) -> Result<()> {
    let parquet_path = data_dir.join("papers-with-abstracts/train.parquet");

    if !parquet_path.exists() {
        warn!("Papers parquet file not found: {:?}", parquet_path);
        return Ok(());
    }

    info!("Loading papers from {:?} (using Arrow columnar API)", parquet_path);

    let file = File::open(&parquet_path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let total_rows = builder.metadata().file_metadata().num_rows() as usize;
    info!("Total papers in file: {}", total_rows);

    // Read in batches using Arrow - much faster than row iteration
    let reader = builder.with_batch_size(batch_size).build()?;

    let mut processed = 0;
    let mut batch_num = 0;

    for batch_result in reader {
        let batch = batch_result?;
        batch_num += 1;

        // Extract columns by index (schema: paper_url=0, arxiv_id=1, title=4, abstract=5, url_abs=7, url_pdf=8)
        let arxiv_id_col = get_string_column(&batch, 1);
        let title_col = get_string_column(&batch, 4);
        let abstract_col = get_string_column(&batch, 5);
        let url_abs_col = get_string_column(&batch, 7);
        let url_pdf_col = get_string_column(&batch, 8);

        if arxiv_id_col.is_none() {
            warn!("Could not get arxiv_id column from batch {}", batch_num);
            continue;
        }

        let arxiv_id_arr = arxiv_id_col.unwrap();
        let num_rows = batch.num_rows();

        // Build vectors for batch insert
        let mut titles: Vec<Option<String>> = Vec::with_capacity(num_rows);
        let mut abstracts: Vec<Option<String>> = Vec::with_capacity(num_rows);
        let mut arxiv_ids: Vec<String> = Vec::with_capacity(num_rows);
        let mut arxiv_urls: Vec<Option<String>> = Vec::with_capacity(num_rows);
        let mut pdf_urls: Vec<Option<String>> = Vec::with_capacity(num_rows);

        for i in 0..num_rows {
            // Skip if arxiv_id is null or empty
            let arxiv_id = if arxiv_id_arr.is_null(i) {
                None
            } else {
                let val = arxiv_id_arr.value(i);
                if val.is_empty() { None } else { Some(val.to_string()) }
            };

            // Get title - skip if null (DB has NOT NULL constraint)
            let title = title_col.and_then(|c| if c.is_null(i) { None } else {
                let t = c.value(i);
                if t.is_empty() { None } else { Some(t.to_string()) }
            });

            match (arxiv_id, title) {
                (Some(id), Some(t)) => {
                    arxiv_ids.push(id);
                    titles.push(Some(t));
                    abstracts.push(abstract_col.and_then(|c| if c.is_null(i) { None } else { Some(c.value(i).to_string()) }));
                    arxiv_urls.push(url_abs_col.and_then(|c| if c.is_null(i) { None } else { Some(c.value(i).to_string()) }));
                    pdf_urls.push(url_pdf_col.and_then(|c| if c.is_null(i) { None } else { Some(c.value(i).to_string()) }));
                }
                _ => {
                    stats.papers_skipped += 1;
                }
            }
        }

        processed += num_rows;

        // Insert batch
        if !arxiv_ids.is_empty() {
            match insert_paper_batch(pool, &titles, &abstracts, &arxiv_ids, &arxiv_urls, &pdf_urls).await {
                Ok(inserted) => {
                    stats.papers_inserted += inserted;
                    stats.papers_skipped += arxiv_ids.len() - inserted;
                }
                Err(e) => {
                    warn!("Error inserting batch {}: {}. Retrying with smaller chunks...", batch_num, e);
                    // Retry in smaller chunks
                    let chunk_size = 100;
                    for chunk_start in (0..arxiv_ids.len()).step_by(chunk_size) {
                        let chunk_end = (chunk_start + chunk_size).min(arxiv_ids.len());
                        match insert_paper_batch(
                            pool,
                            &titles[chunk_start..chunk_end].to_vec(),
                            &abstracts[chunk_start..chunk_end].to_vec(),
                            &arxiv_ids[chunk_start..chunk_end].to_vec(),
                            &arxiv_urls[chunk_start..chunk_end].to_vec(),
                            &pdf_urls[chunk_start..chunk_end].to_vec(),
                        ).await {
                            Ok(inserted) => {
                                stats.papers_inserted += inserted;
                                stats.papers_skipped += (chunk_end - chunk_start) - inserted;
                            }
                            Err(e2) => {
                                warn!("Chunk insert failed: {}. Skipping {} papers.", e2, chunk_end - chunk_start);
                                stats.papers_skipped += chunk_end - chunk_start;
                            }
                        }
                    }
                }
            }
        }

        if batch_num % 10 == 0 || processed >= total_rows {
            info!(
                "Progress: {}/{} papers ({:.1}%) - {} inserted, {} skipped",
                processed, total_rows, (processed as f64 / total_rows as f64) * 100.0,
                stats.papers_inserted, stats.papers_skipped
            );
        }
    }

    info!(
        "Papers complete: {} inserted, {} skipped",
        stats.papers_inserted, stats.papers_skipped
    );
    Ok(())
}

async fn load_datasets(
    pool: &PgPool,
    data_dir: &PathBuf,
    batch_size: usize,
    stats: &mut LoaderStats,
) -> Result<()> {
    let parquet_path = data_dir.join("datasets/train.parquet");

    if !parquet_path.exists() {
        warn!("Datasets parquet file not found: {:?}", parquet_path);
        return Ok(());
    }

    info!("Loading datasets from {:?}", parquet_path);

    let file = File::open(&parquet_path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let total_rows = builder.metadata().file_metadata().num_rows() as usize;
    info!("Total datasets in file: {}", total_rows);

    let reader = builder.with_batch_size(batch_size).build()?;

    let mut processed = 0;

    for batch_result in reader {
        let batch = batch_result?;

        // Schema: name=0, full_name=1, description=2, citation=3, homepage=4
        let name_col = get_string_column(&batch, 0);
        let desc_col = get_string_column(&batch, 2);
        let homepage_col = get_string_column(&batch, 4);

        if name_col.is_none() {
            continue;
        }

        let name_arr = name_col.unwrap();
        let num_rows = batch.num_rows();

        let mut names: Vec<String> = Vec::with_capacity(num_rows);
        let mut descriptions: Vec<Option<String>> = Vec::with_capacity(num_rows);
        let mut homepage_urls: Vec<Option<String>> = Vec::with_capacity(num_rows);

        for i in 0..num_rows {
            if name_arr.is_null(i) {
                continue;
            }
            let name = name_arr.value(i);
            if name.is_empty() {
                continue;
            }

            names.push(name.to_string());
            descriptions.push(desc_col.and_then(|c| if c.is_null(i) { None } else { Some(c.value(i).to_string()) }));
            homepage_urls.push(homepage_col.and_then(|c| if c.is_null(i) { None } else { Some(c.value(i).to_string()) }));
        }

        processed += num_rows;

        if !names.is_empty() {
            let inserted = insert_dataset_batch(pool, &names, &descriptions, &homepage_urls).await?;
            stats.datasets_inserted += inserted;
        }

        info!(
            "Progress: {}/{} datasets ({:.1}%) - {} inserted",
            processed, total_rows, (processed as f64 / total_rows as f64) * 100.0,
            stats.datasets_inserted
        );
    }

    info!("Datasets complete: {} inserted", stats.datasets_inserted);
    Ok(())
}

async fn load_links(
    pool: &PgPool,
    data_dir: &PathBuf,
    batch_size: usize,
    stats: &mut LoaderStats,
) -> Result<()> {
    let parquet_path = data_dir.join("links-between-paper-and-code/train.parquet");

    if !parquet_path.exists() {
        warn!("Links parquet file not found: {:?}", parquet_path);
        return Ok(());
    }

    info!("Loading code links from {:?}", parquet_path);

    let file = File::open(&parquet_path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let total_rows = builder.metadata().file_metadata().num_rows() as usize;
    info!("Total links in file: {}", total_rows);

    let reader = builder.with_batch_size(batch_size).build()?;

    let mut processed = 0;

    for batch_result in reader {
        let batch = batch_result?;

        // Schema: paper_arxiv_id=2, repo_url=5, framework=9
        let arxiv_col = get_string_column(&batch, 2);
        let repo_col = get_string_column(&batch, 5);
        let framework_col = get_string_column(&batch, 9);

        if arxiv_col.is_none() || repo_col.is_none() {
            continue;
        }

        let arxiv_arr = arxiv_col.unwrap();
        let repo_arr = repo_col.unwrap();
        let num_rows = batch.num_rows();

        let mut arxiv_ids: Vec<String> = Vec::with_capacity(num_rows);
        let mut repo_urls: Vec<String> = Vec::with_capacity(num_rows);
        let mut frameworks: Vec<Option<String>> = Vec::with_capacity(num_rows);

        for i in 0..num_rows {
            if arxiv_arr.is_null(i) || repo_arr.is_null(i) {
                continue;
            }
            let arxiv_id = arxiv_arr.value(i);
            let repo_url = repo_arr.value(i);
            if arxiv_id.is_empty() || repo_url.is_empty() {
                continue;
            }

            arxiv_ids.push(arxiv_id.to_string());
            repo_urls.push(repo_url.to_string());
            frameworks.push(framework_col.and_then(|c| if c.is_null(i) { None } else { Some(c.value(i).to_string()) }));
        }

        processed += num_rows;

        if !arxiv_ids.is_empty() {
            let inserted = insert_link_batch(pool, &arxiv_ids, &repo_urls, &frameworks).await?;
            stats.links_inserted += inserted;
        }

        info!(
            "Progress: {}/{} links ({:.1}%) - {} inserted",
            processed, total_rows, (processed as f64 / total_rows as f64) * 100.0,
            stats.links_inserted
        );
    }

    info!("Links complete: {} inserted", stats.links_inserted);
    Ok(())
}

fn print_stats(stats: &LoaderStats) {
    info!("=== Loading Statistics ===");
    info!(
        "Papers: {} inserted, {} skipped",
        stats.papers_inserted, stats.papers_skipped
    );
    info!("Datasets: {} inserted", stats.datasets_inserted);
    info!("Links: {} inserted", stats.links_inserted);
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

    info!("Starting Optimized Data Loader (Arrow columnar + smaller batches)...");
    info!("Data directory: {:?}", args.data_dir);
    info!("Batch size: {}", args.batch_size);

    // Connect to database with shorter timeouts for serverless
    let database_url = env::var("POSTGRES_URI").context("POSTGRES_URI must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;
    info!("Connected to database");

    let mut stats = LoaderStats::default();

    // Load data based on --only flag or all
    match args.only.as_deref() {
        Some("papers") => {
            load_papers(&pool, &args.data_dir, args.batch_size, &mut stats).await?;
        }
        Some("datasets") => {
            load_datasets(&pool, &args.data_dir, args.batch_size, &mut stats).await?;
        }
        Some("links") => {
            load_links(&pool, &args.data_dir, args.batch_size, &mut stats).await?;
        }
        Some(other) => {
            warn!("Unknown dataset: {}. Use: papers, datasets, links", other);
        }
        None => {
            // Load all in order
            load_papers(&pool, &args.data_dir, args.batch_size, &mut stats).await?;
            load_datasets(&pool, &args.data_dir, args.batch_size, &mut stats).await?;
            load_links(&pool, &args.data_dir, args.batch_size, &mut stats).await?;
        }
    }

    print_stats(&stats);
    info!("Loading complete.");

    Ok(())
}
