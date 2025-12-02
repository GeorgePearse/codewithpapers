//! Process Paper Submissions
//!
//! Inserts validated YAML submissions into the PostgreSQL database.
//! Each submission is processed in a single transaction (all-or-nothing).
//! Generates an audit log for tracking.
//!
//! Usage:
//!     process_submission --audit-log audit.json
//!     process_submission --files submission1.yaml submission2.yaml --audit-log audit.json

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use clap::Parser;
use dotenvy::dotenv;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Process YAML paper submissions into database",
    long_about = "Validates and inserts paper submissions from YAML files into PostgreSQL.\n\
                  Each submission is processed atomically - all or nothing."
)]
struct Args {
    /// Specific files to process (default: all in submissions/)
    #[arg(long)]
    files: Option<Vec<PathBuf>>,

    /// Directory containing submission files
    #[arg(long, default_value = "submissions")]
    submissions_dir: PathBuf,

    /// Path for audit log output (JSON)
    #[arg(long, required = true)]
    audit_log: PathBuf,

    /// Dry run - validate only, don't insert
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

// =============================================================================
// Submission Models (YAML input format)
// =============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PaperSubmission {
    pub title: String,
    pub arxiv_id: String,
    #[serde(default)]
    pub r#abstract: Option<String>,
    #[serde(default)]
    pub arxiv_url: Option<String>,
    #[serde(default)]
    pub pdf_url: Option<String>,
    #[serde(default)]
    pub published_date: Option<NaiveDate>,
    #[serde(default)]
    pub authors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ImplementationSubmission {
    pub github_url: String,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default)]
    pub is_official: bool,
    #[serde(default)]
    pub stars: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkResultSubmission {
    pub dataset_name: String,
    pub task: String,
    pub metric_name: String,
    pub metric_value: Decimal,
    #[serde(default)]
    pub extra_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FullSubmission {
    pub paper: PaperSubmission,
    #[serde(default)]
    pub implementations: Option<Vec<ImplementationSubmission>>,
    #[serde(default)]
    pub benchmark_results: Option<Vec<BenchmarkResultSubmission>>,
}

// =============================================================================
// Audit Log Types
// =============================================================================

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InsertionStatus {
    Success,
    Duplicate,
    Failed,
    Skipped,
    RolledBack,
}

#[derive(Debug, Serialize, Clone)]
pub struct InsertionRecord {
    pub table: String,
    pub identifier: String,
    pub status: InsertionStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuditEntry {
    pub file_path: String,
    pub timestamp: String,
    pub commit_sha: String,
    pub overall_status: InsertionStatus,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub error_message: String,
    pub rollback_performed: bool,
    pub records: Vec<InsertionRecord>,
}

impl AuditEntry {
    fn new(file_path: &str, commit_sha: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            commit_sha: commit_sha.to_string(),
            overall_status: InsertionStatus::Skipped,
            error_message: String::new(),
            rollback_performed: false,
            records: Vec::new(),
        }
    }
}

// =============================================================================
// Database Insertion
// =============================================================================

async fn insert_paper(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    paper: &PaperSubmission,
) -> Result<(Uuid, bool)> {
    let authors_json = paper
        .authors
        .as_ref()
        .map(|a| serde_json::to_value(a).unwrap());

    // Use UPSERT to handle duplicates gracefully
    let row: (Uuid, bool) = sqlx::query_as(
        r#"
        INSERT INTO papers (title, abstract, arxiv_id, arxiv_url, pdf_url, published_date, authors)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (arxiv_id) DO UPDATE SET
            title = EXCLUDED.title,
            abstract = COALESCE(EXCLUDED.abstract, papers.abstract),
            arxiv_url = COALESCE(EXCLUDED.arxiv_url, papers.arxiv_url),
            pdf_url = COALESCE(EXCLUDED.pdf_url, papers.pdf_url),
            published_date = COALESCE(EXCLUDED.published_date, papers.published_date),
            authors = COALESCE(EXCLUDED.authors, papers.authors),
            updated_at = NOW()
        RETURNING id, (xmax = 0)
        "#,
    )
    .bind(&paper.title)
    .bind(&paper.r#abstract)
    .bind(&paper.arxiv_id)
    .bind(&paper.arxiv_url)
    .bind(&paper.pdf_url)
    .bind(paper.published_date)
    .bind(&authors_json)
    .fetch_one(&mut **tx)
    .await
    .context("Failed to insert paper")?;

    Ok(row)
}

async fn insert_implementation(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    impl_: &ImplementationSubmission,
    paper_id: Uuid,
) -> Result<(Uuid, bool)> {
    let row: (Uuid, bool) = sqlx::query_as(
        r#"
        INSERT INTO implementations (paper_id, github_url, framework, is_official, stars)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (paper_id, github_url) DO UPDATE SET
            framework = COALESCE(EXCLUDED.framework, implementations.framework),
            is_official = EXCLUDED.is_official,
            stars = COALESCE(EXCLUDED.stars, implementations.stars),
            updated_at = NOW()
        RETURNING id, (xmax = 0)
        "#,
    )
    .bind(paper_id)
    .bind(&impl_.github_url)
    .bind(&impl_.framework)
    .bind(impl_.is_official)
    .bind(impl_.stars)
    .fetch_one(&mut **tx)
    .await
    .context("Failed to insert implementation")?;

    Ok(row)
}

async fn insert_benchmark_result(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    result: &BenchmarkResultSubmission,
    paper_id: Uuid,
) -> Result<(Uuid, bool)> {
    // First, get or create dataset
    let (dataset_id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO datasets (name)
        VALUES ($1)
        ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(&result.dataset_name)
    .fetch_one(&mut **tx)
    .await
    .context("Failed to get/create dataset")?;

    // Get or create benchmark
    let benchmark_name = format!("{} - {}", result.dataset_name, result.task);
    let (benchmark_id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO benchmarks (name, dataset_id, task)
        VALUES ($1, $2, $3)
        ON CONFLICT (name, dataset_id) DO UPDATE SET task = EXCLUDED.task
        RETURNING id
        "#,
    )
    .bind(&benchmark_name)
    .bind(dataset_id)
    .bind(&result.task)
    .fetch_one(&mut **tx)
    .await
    .context("Failed to get/create benchmark")?;

    // Insert the result
    let metric_value_f64 = result
        .metric_value
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);
    let metric_value_decimal = sqlx::types::Decimal::try_from(metric_value_f64).unwrap();

    let row: (Uuid, bool) = sqlx::query_as(
        r#"
        INSERT INTO benchmark_results (paper_id, benchmark_id, metric_name, metric_value, extra_data)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (paper_id, benchmark_id, metric_name) DO UPDATE SET
            metric_value = EXCLUDED.metric_value,
            extra_data = COALESCE(EXCLUDED.extra_data, benchmark_results.extra_data)
        RETURNING id, (xmax = 0)
        "#,
    )
    .bind(paper_id)
    .bind(benchmark_id)
    .bind(&result.metric_name)
    .bind(metric_value_decimal)
    .bind(&result.extra_data)
    .fetch_one(&mut **tx)
    .await
    .context("Failed to insert benchmark result")?;

    Ok(row)
}

async fn process_submission(
    pool: &PgPool,
    submission: &FullSubmission,
    file_path: &str,
    commit_sha: &str,
) -> AuditEntry {
    let mut audit = AuditEntry::new(file_path, commit_sha);

    // Start transaction
    let tx_result = pool.begin().await;
    let mut tx = match tx_result {
        Ok(tx) => tx,
        Err(e) => {
            audit.overall_status = InsertionStatus::Failed;
            audit.error_message = format!("Failed to start transaction: {}", e);
            return audit;
        }
    };

    // Insert paper
    let paper_result = insert_paper(&mut tx, &submission.paper).await;
    let paper_id = match paper_result {
        Ok((id, inserted)) => {
            audit.records.push(InsertionRecord {
                table: "papers".to_string(),
                identifier: submission.paper.arxiv_id.clone(),
                status: if inserted {
                    InsertionStatus::Success
                } else {
                    InsertionStatus::Duplicate
                },
                message: if inserted {
                    "Inserted new paper".to_string()
                } else {
                    "Updated existing paper".to_string()
                },
                db_id: Some(id.to_string()),
            });
            id
        }
        Err(e) => {
            audit.records.push(InsertionRecord {
                table: "papers".to_string(),
                identifier: submission.paper.arxiv_id.clone(),
                status: InsertionStatus::Failed,
                message: e.to_string(),
                db_id: None,
            });
            audit.overall_status = InsertionStatus::RolledBack;
            audit.error_message = format!("Paper insertion failed: {}", e);
            audit.rollback_performed = true;
            let _ = tx.rollback().await;
            return audit;
        }
    };

    // Insert implementations
    if let Some(ref impls) = submission.implementations {
        for impl_ in impls {
            match insert_implementation(&mut tx, impl_, paper_id).await {
                Ok((id, inserted)) => {
                    audit.records.push(InsertionRecord {
                        table: "implementations".to_string(),
                        identifier: impl_.github_url.clone(),
                        status: if inserted {
                            InsertionStatus::Success
                        } else {
                            InsertionStatus::Duplicate
                        },
                        message: if inserted {
                            "Inserted".to_string()
                        } else {
                            "Updated existing".to_string()
                        },
                        db_id: Some(id.to_string()),
                    });
                }
                Err(e) => {
                    audit.records.push(InsertionRecord {
                        table: "implementations".to_string(),
                        identifier: impl_.github_url.clone(),
                        status: InsertionStatus::Failed,
                        message: e.to_string(),
                        db_id: None,
                    });
                    audit.overall_status = InsertionStatus::RolledBack;
                    audit.error_message = format!("Implementation insertion failed: {}", e);
                    audit.rollback_performed = true;
                    let _ = tx.rollback().await;
                    return audit;
                }
            }
        }
    }

    // Insert benchmark results
    if let Some(ref results) = submission.benchmark_results {
        for result in results {
            let identifier = format!(
                "{}/{}/{}",
                result.dataset_name, result.task, result.metric_name
            );
            match insert_benchmark_result(&mut tx, result, paper_id).await {
                Ok((id, inserted)) => {
                    audit.records.push(InsertionRecord {
                        table: "benchmark_results".to_string(),
                        identifier,
                        status: if inserted {
                            InsertionStatus::Success
                        } else {
                            InsertionStatus::Duplicate
                        },
                        message: if inserted {
                            "Inserted".to_string()
                        } else {
                            "Updated existing".to_string()
                        },
                        db_id: Some(id.to_string()),
                    });
                }
                Err(e) => {
                    audit.records.push(InsertionRecord {
                        table: "benchmark_results".to_string(),
                        identifier,
                        status: InsertionStatus::Failed,
                        message: e.to_string(),
                        db_id: None,
                    });
                    audit.overall_status = InsertionStatus::RolledBack;
                    audit.error_message = format!("Benchmark result insertion failed: {}", e);
                    audit.rollback_performed = true;
                    let _ = tx.rollback().await;
                    return audit;
                }
            }
        }
    }

    // Commit transaction
    match tx.commit().await {
        Ok(_) => {
            audit.overall_status = InsertionStatus::Success;
            info!("Successfully processed submission from {}", file_path);
        }
        Err(e) => {
            audit.overall_status = InsertionStatus::Failed;
            audit.error_message = format!("Failed to commit transaction: {}", e);
            error!("Failed to commit: {}", e);
        }
    }

    audit
}

// =============================================================================
// File Discovery
// =============================================================================

fn find_yaml_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if (ext == "yaml" || ext == "yml")
                        && !path
                            .file_name()
                            .map(|n| n.to_string_lossy().starts_with("example"))
                            .unwrap_or(false)
                        && !path
                            .file_name()
                            .map(|n| n.to_string_lossy().starts_with('_'))
                            .unwrap_or(false)
                    {
                        files.push(path);
                    }
                }
            }
        }
    }

    files
}

fn parse_submission(path: &PathBuf) -> Result<FullSubmission> {
    let content = fs::read_to_string(path).context("Failed to read file")?;
    let submission: FullSubmission =
        serde_yaml::from_str(&content).context("Failed to parse YAML")?;
    Ok(submission)
}

// =============================================================================
// Main
// =============================================================================

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

    // Get commit SHA for audit trail
    let commit_sha = env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string());

    // Find files to process
    let files_to_process: Vec<PathBuf> = if let Some(files) = args.files {
        files
    } else if args.submissions_dir.exists() {
        find_yaml_files(&args.submissions_dir)
    } else {
        info!("Submissions directory not found: {:?}", args.submissions_dir);
        // Write empty audit log
        fs::write(&args.audit_log, "[]")?;
        return Ok(());
    };

    if files_to_process.is_empty() {
        info!("No submission files to process");
        fs::write(&args.audit_log, "[]")?;
        return Ok(());
    }

    info!("Processing {} submission(s)...", files_to_process.len());

    let mut audit_entries: Vec<AuditEntry> = Vec::new();

    if args.dry_run {
        info!("Dry run mode - validating only");
        for path in &files_to_process {
            let path_str = path.display().to_string();
            let mut audit = AuditEntry::new(&path_str, &commit_sha);

            match parse_submission(path) {
                Ok(_) => {
                    audit.overall_status = InsertionStatus::Success;
                    info!("Valid: {}", path_str);
                }
                Err(e) => {
                    audit.overall_status = InsertionStatus::Failed;
                    audit.error_message = e.to_string();
                    error!("Invalid: {} - {}", path_str, e);
                }
            }
            audit_entries.push(audit);
        }
    } else {
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

        // Process each file
        for path in &files_to_process {
            let path_str = path.display().to_string();

            // Parse submission
            let submission = match parse_submission(path) {
                Ok(s) => s,
                Err(e) => {
                    let mut audit = AuditEntry::new(&path_str, &commit_sha);
                    audit.overall_status = InsertionStatus::Failed;
                    audit.error_message = format!("Failed to parse: {}", e);
                    audit_entries.push(audit);
                    error!("Failed to parse {}: {}", path_str, e);
                    continue;
                }
            };

            // Process submission
            let audit = process_submission(&pool, &submission, &path_str, &commit_sha).await;
            audit_entries.push(audit);
        }
    }

    // Write audit log
    let audit_json = serde_json::to_string_pretty(&audit_entries)?;
    fs::write(&args.audit_log, &audit_json)?;
    info!("Audit log written to {:?}", args.audit_log);

    // Summary
    let success_count = audit_entries
        .iter()
        .filter(|a| matches!(a.overall_status, InsertionStatus::Success | InsertionStatus::Duplicate))
        .count();
    let failed_count = audit_entries.len() - success_count;

    info!(
        "Results: {} successful, {} failed",
        success_count, failed_count
    );

    if failed_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
