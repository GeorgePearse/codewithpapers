//! YAML Submission Validator
//!
//! Validates paper submissions in YAML format against the database schema.
//! Used as a pre-commit hook and in CI to ensure submissions are valid before merge.
//!
//! Usage:
//!     validate_submission submissions/my-paper.yaml
//!     validate_submission submissions/  # validates all YAML files in directory

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Validate YAML paper submissions",
    long_about = "Validates paper submissions in YAML format against the expected schema.\n\
                  Can validate individual files or all YAML files in a directory."
)]
struct Args {
    /// Path to a YAML file or directory containing YAML files
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Output format (human, json, github)
    #[arg(short, long, default_value = "human")]
    format: OutputFormat,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Github,
}

// =============================================================================
// Submission Models (YAML input format)
// =============================================================================

/// Paper submission data from YAML
#[derive(Debug, Deserialize, Serialize)]
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

/// Implementation submission data from YAML
#[derive(Debug, Deserialize, Serialize)]
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

/// Benchmark result submission data from YAML
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkResultSubmission {
    pub dataset_name: String,
    pub task: String,
    pub metric_name: String,
    pub metric_value: Decimal,
    #[serde(default)]
    pub extra_data: Option<serde_json::Value>,
}

/// Full submission containing a paper and optionally related data
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FullSubmission {
    pub paper: PaperSubmission,
    #[serde(default)]
    pub implementations: Option<Vec<ImplementationSubmission>>,
    #[serde(default)]
    pub benchmark_results: Option<Vec<BenchmarkResultSubmission>>,
}

// =============================================================================
// Validation Logic
// =============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub field: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub file_path: String,
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            valid: false,
            issues: Vec::new(),
        }
    }

    fn add_error(&mut self, field: &str, message: &str, suggestion: Option<&str>) {
        self.issues.push(ValidationIssue {
            severity: IssueSeverity::Error,
            field: field.to_string(),
            message: message.to_string(),
            suggestion: suggestion.map(|s| s.to_string()),
        });
    }

    fn add_warning(&mut self, field: &str, message: &str, suggestion: Option<&str>) {
        self.issues.push(ValidationIssue {
            severity: IssueSeverity::Warning,
            field: field.to_string(),
            message: message.to_string(),
            suggestion: suggestion.map(|s| s.to_string()),
        });
    }

    fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == IssueSeverity::Error)
    }
}

/// Validate an arXiv ID format
fn validate_arxiv_id(id: &str) -> Result<(), String> {
    // Standard format: YYMM.NNNNN with optional version
    let standard_pattern = regex::Regex::new(r"^\d{4}\.\d{4,5}(v\d+)?$").unwrap();
    // Old format: category/NNNNNNN
    let old_pattern = regex::Regex::new(r"^[a-z-]+(\.[A-Z]{2})?/\d{7}$").unwrap();

    if standard_pattern.is_match(id) || old_pattern.is_match(id) {
        Ok(())
    } else {
        Err(format!(
            "Invalid arXiv ID format: '{}'. Expected format like '2301.12345', '2301.12345v2', or 'cs.CV/0601001'",
            id
        ))
    }
}

/// Validate a GitHub URL
fn validate_github_url(url: &str) -> Result<(), String> {
    if !url.contains("github.com") {
        return Err("URL must be a github.com URL".to_string());
    }

    let pattern = regex::Regex::new(r"https://github\.com/[\w.-]+/[\w.-]+").unwrap();
    if !pattern.is_match(url) {
        return Err("URL must follow format: https://github.com/owner/repo".to_string());
    }

    Ok(())
}

/// Validate a URL (basic check)
fn validate_url(url: &str, field_name: &str) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("{} must start with http:// or https://", field_name));
    }
    Ok(())
}

/// Validate a single submission file
fn validate_file(path: &PathBuf) -> ValidationResult {
    let path_str = path.display().to_string();
    let mut result = ValidationResult::new(&path_str);

    // Read file
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            result.add_error("file", &format!("Cannot read file: {}", e), None);
            return result;
        }
    };

    // Parse YAML
    let submission: FullSubmission = match serde_yaml::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("YAML parse error: {}", e);
            let suggestion = if msg.contains("unknown field") {
                Some("Check for typos in field names")
            } else if msg.contains("missing field") {
                Some("Add the required field")
            } else {
                Some("Check YAML syntax and indentation")
            };
            result.add_error("yaml", &msg, suggestion);
            return result;
        }
    };

    // Validate paper fields
    let paper = &submission.paper;

    // Title validation
    if paper.title.trim().is_empty() {
        result.add_error("paper.title", "Title cannot be empty", None);
    } else if paper.title.len() < 5 {
        result.add_error(
            "paper.title",
            "Title is too short (minimum 5 characters)",
            None,
        );
    } else if paper.title.len() > 500 {
        result.add_error(
            "paper.title",
            "Title is too long (maximum 500 characters)",
            None,
        );
    }

    // arXiv ID validation
    if let Err(e) = validate_arxiv_id(&paper.arxiv_id) {
        result.add_error("paper.arxiv_id", &e, None);
    }

    // URL validations (if provided)
    if let Some(ref url) = paper.arxiv_url {
        if let Err(e) = validate_url(url, "arxiv_url") {
            result.add_error("paper.arxiv_url", &e, None);
        } else if !url.contains("arxiv.org") {
            result.add_warning(
                "paper.arxiv_url",
                "arxiv_url should point to arxiv.org",
                None,
            );
        }
    }

    if let Some(ref url) = paper.pdf_url {
        if let Err(e) = validate_url(url, "pdf_url") {
            result.add_error("paper.pdf_url", &e, None);
        }
    }

    // Date validation
    if let Some(date) = paper.published_date {
        let today = chrono::Local::now().date_naive();
        if date > today {
            result.add_error(
                "paper.published_date",
                "Published date cannot be in the future",
                None,
            );
        }
    }

    // Validate implementations
    if let Some(ref impls) = submission.implementations {
        for (i, impl_) in impls.iter().enumerate() {
            let field_prefix = format!("implementations[{}]", i);

            if let Err(e) = validate_github_url(&impl_.github_url) {
                result.add_error(&format!("{}.github_url", field_prefix), &e, None);
            }

            // Validate framework if provided
            if let Some(ref fw) = impl_.framework {
                let valid_frameworks = [
                    "pytorch",
                    "tensorflow",
                    "jax",
                    "keras",
                    "sklearn",
                    "other",
                ];
                if !valid_frameworks.contains(&fw.to_lowercase().as_str()) {
                    result.add_warning(
                        &format!("{}.framework", field_prefix),
                        &format!(
                            "Unknown framework '{}'. Expected one of: {:?}",
                            fw, valid_frameworks
                        ),
                        None,
                    );
                }
            }
        }
    }

    // Validate benchmark results
    if let Some(ref results) = submission.benchmark_results {
        for (i, res) in results.iter().enumerate() {
            let field_prefix = format!("benchmark_results[{}]", i);

            if res.dataset_name.trim().is_empty() {
                result.add_error(
                    &format!("{}.dataset_name", field_prefix),
                    "Dataset name cannot be empty",
                    None,
                );
            }

            if res.task.trim().is_empty() {
                result.add_error(
                    &format!("{}.task", field_prefix),
                    "Task cannot be empty",
                    None,
                );
            }

            if res.metric_name.trim().is_empty() {
                result.add_error(
                    &format!("{}.metric_name", field_prefix),
                    "Metric name cannot be empty",
                    None,
                );
            }
        }
    }

    // Add warnings for missing optional but recommended fields
    if paper.r#abstract.is_none() {
        result.add_warning(
            "paper.abstract",
            "No abstract provided",
            Some("Consider adding an abstract for better discoverability"),
        );
    }

    if paper.authors.is_none() || paper.authors.as_ref().map(|a| a.is_empty()).unwrap_or(true) {
        result.add_warning(
            "paper.authors",
            "No authors listed",
            Some("Consider adding the author list"),
        );
    }

    if paper.published_date.is_none() {
        result.add_warning(
            "paper.published_date",
            "No publication date",
            Some("Add the publication date in YYYY-MM-DD format"),
        );
    }

    if submission.implementations.is_none()
        || submission
            .implementations
            .as_ref()
            .map(|i| i.is_empty())
            .unwrap_or(true)
    {
        result.add_warning(
            "implementations",
            "No implementations linked",
            Some("Add code implementations when available"),
        );
    }

    // Set valid flag based on errors
    result.valid = !result.has_errors();
    result
}

/// Find all YAML files in a directory (non-recursively)
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

// =============================================================================
// Output Formatting
// =============================================================================

fn print_human_output(results: &[ValidationResult]) {
    const GREEN: &str = "\x1b[32m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";

    for result in results {
        println!("\n{}Validating: {}{}", BOLD, result.file_path, RESET);

        if result.valid {
            println!("  {}[VALID]{} Schema validation passed", GREEN, RESET);
        } else {
            println!("  {}[INVALID]{} Validation failed", RED, RESET);
        }

        for issue in &result.issues {
            let (prefix, color) = match issue.severity {
                IssueSeverity::Error => ("[ERROR]", RED),
                IssueSeverity::Warning => ("[WARNING]", YELLOW),
            };

            println!(
                "  {}{}{} {}: {}",
                color, prefix, RESET, issue.field, issue.message
            );

            if let Some(ref suggestion) = issue.suggestion {
                println!("          {}", suggestion);
            }
        }
    }

    // Summary
    let total = results.len();
    let valid = results.iter().filter(|r| r.valid).count();
    let invalid = total - valid;

    println!("\n{}", "=".repeat(60));
    println!("Summary: {} file(s) checked", total);
    println!("  Valid: {}", valid);
    println!("  Invalid: {}", invalid);
    println!("{}", "=".repeat(60));

    if invalid == 0 {
        println!("\n{}All submissions are valid!{}", GREEN, RESET);
    } else {
        println!(
            "\n{}Please fix the errors above and try again.{}",
            RED, RESET
        );
    }
}

fn print_json_output(results: &[ValidationResult]) {
    let output = serde_json::to_string_pretty(&results).unwrap();
    println!("{}", output);
}

fn print_github_output(results: &[ValidationResult]) {
    // Print GitHub Actions workflow commands
    for result in results {
        for issue in &result.issues {
            let level = match issue.severity {
                IssueSeverity::Error => "error",
                IssueSeverity::Warning => "warning",
            };
            println!(
                "::{}file={}::{}:{}",
                level, result.file_path, issue.field, issue.message
            );
        }
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() -> Result<()> {
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

    // Collect all files to validate
    let mut files_to_validate: Vec<PathBuf> = Vec::new();

    for path in &args.paths {
        if path.is_dir() {
            let yaml_files = find_yaml_files(path);
            if yaml_files.is_empty() {
                warn!("No YAML files found in directory: {:?}", path);
            }
            files_to_validate.extend(yaml_files);
        } else if path.is_file() {
            files_to_validate.push(path.clone());
        } else {
            error!("Path does not exist: {:?}", path);
        }
    }

    if files_to_validate.is_empty() {
        info!("No files to validate");
        return Ok(());
    }

    info!("Validating {} file(s)...", files_to_validate.len());

    // Validate all files
    let results: Vec<ValidationResult> = files_to_validate.iter().map(validate_file).collect();

    // Output results
    match args.format {
        OutputFormat::Human => print_human_output(&results),
        OutputFormat::Json => print_json_output(&results),
        OutputFormat::Github => {
            print_github_output(&results);
            print_human_output(&results);
        }
    }

    // Exit with error if any files are invalid
    let all_valid = results.iter().all(|r| r.valid);
    if !all_valid {
        std::process::exit(1);
    }

    Ok(())
}
