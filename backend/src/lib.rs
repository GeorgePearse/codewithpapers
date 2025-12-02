use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub mod search;

// ============================================================================
// Response Types
// ============================================================================

#[derive(Serialize)]
pub struct Message {
    pub message: String,
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
}

// ============================================================================
// Database Models
// ============================================================================

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug, Clone)]
pub struct Paper {
    pub id: uuid::Uuid,
    pub title: String,
    pub r#abstract: Option<String>,
    pub arxiv_id: Option<String>,
    pub arxiv_url: Option<String>,
    pub pdf_url: Option<String>,
    pub published_date: Option<chrono::NaiveDate>,
    pub authors: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct PaperSummary {
    pub id: uuid::Uuid,
    pub title: String,
    pub arxiv_id: Option<String>,
    pub published_date: Option<chrono::NaiveDate>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct Dataset {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub modalities: Option<Vec<String>>,
    pub task_categories: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub size: Option<String>,
    pub homepage_url: Option<String>,
    pub github_url: Option<String>,
    pub paper_url: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct Benchmark {
    pub id: uuid::Uuid,
    pub name: String,
    pub dataset_id: Option<uuid::Uuid>,
    pub task: String,
    pub description: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct Implementation {
    pub id: uuid::Uuid,
    pub paper_id: Option<uuid::Uuid>,
    pub github_url: String,
    pub framework: Option<String>,
    pub stars: Option<i32>,
    pub is_official: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct BenchmarkResult {
    pub id: uuid::Uuid,
    pub paper_id: Option<uuid::Uuid>,
    pub benchmark_id: Option<uuid::Uuid>,
    pub implementation_id: Option<uuid::Uuid>,
    pub metric_name: String,
    pub metric_value: rust_decimal::Decimal,
    pub extra_data: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub order_by: Option<String>,
    pub order: Option<String>,
    pub search: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(20),
            offset: Some(0),
            order_by: None,
            order: Some("desc".to_string()),
            search: None,
        }
    }
}

// ============================================================================
// Joined Response Types
// ============================================================================

#[derive(Serialize, Debug)]
pub struct PaperWithImplementations {
    #[serde(flatten)]
    pub paper: Paper,
    pub implementations: Vec<Implementation>,
}

#[derive(Serialize, Debug)]
pub struct BenchmarkWithDataset {
    #[serde(flatten)]
    pub benchmark: Benchmark,
    pub dataset: Option<Dataset>,
}

#[derive(Serialize, Debug)]
pub struct StatsResponse {
    pub papers_count: i64,
    pub datasets_count: i64,
    pub benchmarks_count: i64,
    pub implementations_count: i64,
}

// ============================================================================
// App State
// ============================================================================

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub search_index: Option<Arc<search::SearchIndex>>,
}

// ============================================================================
// Router Setup
// ============================================================================

pub fn create_app(pool: Pool<Postgres>, search_index: Option<Arc<search::SearchIndex>>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState { pool, search_index };

    Router::new()
        // Health & Stats
        .route("/", get(root))
        .route("/api/health", get(health_check))
        .route("/api/stats", get(get_stats))
        // Papers
        .route("/api/papers", get(get_papers))
        .route("/api/papers/{id}", get(get_paper_by_id))
        // Datasets
        .route("/api/datasets", get(get_datasets))
        .route("/api/datasets/{id}", get(get_dataset_by_id))
        // Benchmarks
        .route("/api/benchmarks", get(get_benchmarks))
        .route("/api/benchmarks/{id}", get(get_benchmark_by_id))
        // Implementations
        .route("/api/implementations", get(get_implementations))
        .route("/api/implementations/{id}", get(get_implementation_by_id))
        // Benchmark Results
        .route("/api/benchmark-results", get(get_benchmark_results))
        .layer(cors)
        .with_state(state)
}

// ============================================================================
// Handlers: Health & Stats
// ============================================================================

async fn root() -> &'static str {
    "CodeWithPapers API - v0.1.0"
}

async fn health_check() -> Json<Message> {
    Json(Message {
        message: "Backend is running!".to_string(),
    })
}

async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>, (StatusCode, Json<ApiError>)> {
    let papers_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM papers")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: e.to_string(),
                }),
            )
        })?;

    let datasets_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM datasets")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: e.to_string(),
                }),
            )
        })?;

    let benchmarks_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM benchmarks")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: e.to_string(),
                }),
            )
        })?;

    let implementations_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM implementations")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(StatsResponse {
        papers_count: papers_count.0,
        datasets_count: datasets_count.0,
        benchmarks_count: benchmarks_count.0,
        implementations_count: implementations_count.0,
    }))
}

// ============================================================================
// Handlers: Papers
// ============================================================================

async fn get_papers(
    State(state): State<AppState>,
    Query(params): Query<search::SearchParams>,
) -> Result<Json<search::SearchResponse<Paper>>, (StatusCode, Json<ApiError>)> {
    let limit = params.limit.unwrap_or(20).min(100) as usize;
    let offset = params.offset.unwrap_or(0) as usize;
    let order = if params.order.as_deref() == Some("asc") {
        "ASC"
    } else {
        "DESC"
    };

    // If search query provided and Tantivy index available, use full-text search
    if let Some(query_str) = params.get_query() {
        if !query_str.trim().is_empty() {
            if let Some(ref search_index) = state.search_index {
                return search_papers_tantivy(&state, search_index, query_str, &params, limit, offset).await;
            }
            // Fall back to PostgreSQL ILIKE if no Tantivy index
            return search_papers_postgres(&state, query_str, &params, limit, offset, order).await;
        }
    }

    // No search query - browse papers from PostgreSQL
    browse_papers_postgres(&state, limit, offset, order).await
}

/// Search papers using Tantivy full-text search
async fn search_papers_tantivy(
    state: &AppState,
    search_index: &search::SearchIndex,
    query_str: &str,
    params: &search::SearchParams,
    limit: usize,
    offset: usize,
) -> Result<Json<search::SearchResponse<Paper>>, (StatusCode, Json<ApiError>)> {
    // Execute Tantivy search
    let search_result = search::query::search_papers(search_index, query_str, params, limit, offset)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Search failed: {}", e),
                }),
            )
        })?;

    if search_result.paper_ids.is_empty() {
        return Ok(Json(search::SearchResponse {
            papers: vec![],
            total_hits: 0,
            facets: search_result.facets,
        }));
    }

    // Fetch full paper data from PostgreSQL, preserving search order
    let papers = fetch_papers_by_ids(&state.pool, &search_result.paper_ids).await?;

    Ok(Json(search::SearchResponse {
        papers,
        total_hits: search_result.total_hits,
        facets: search_result.facets,
    }))
}

/// Fetch papers by IDs from PostgreSQL, preserving order
async fn fetch_papers_by_ids(
    pool: &Pool<Postgres>,
    ids: &[uuid::Uuid],
) -> Result<Vec<Paper>, (StatusCode, Json<ApiError>)> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    // Fetch all papers by IDs
    let papers: Vec<Paper> = sqlx::query_as(
        r#"
        SELECT id, title, abstract, arxiv_id, arxiv_url, pdf_url,
               published_date, authors, created_at, updated_at
        FROM papers
        WHERE id = ANY($1)
        "#,
    )
    .bind(ids)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })?;

    // Reorder to match search result order
    let paper_map: std::collections::HashMap<uuid::Uuid, Paper> =
        papers.into_iter().map(|p| (p.id, p)).collect();

    let ordered_papers: Vec<Paper> = ids
        .iter()
        .filter_map(|id| paper_map.get(id).cloned())
        .collect();

    Ok(ordered_papers)
}

/// Search papers using PostgreSQL ILIKE (fallback)
async fn search_papers_postgres(
    state: &AppState,
    query_str: &str,
    _params: &search::SearchParams,
    limit: usize,
    offset: usize,
    order: &str,
) -> Result<Json<search::SearchResponse<Paper>>, (StatusCode, Json<ApiError>)> {
    let search_pattern = format!("%{}%", query_str);

    let papers: Vec<Paper> = sqlx::query_as(&format!(
        r#"
        SELECT id, title, abstract, arxiv_id, arxiv_url, pdf_url,
               published_date, authors, created_at, updated_at
        FROM papers
        WHERE title ILIKE $1 OR abstract ILIKE $1
        ORDER BY published_date {} NULLS LAST
        LIMIT $2 OFFSET $3
        "#,
        order
    ))
    .bind(&search_pattern)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(Json(search::SearchResponse {
        papers,
        total_hits: 0, // PostgreSQL fallback doesn't provide total count
        facets: None,
    }))
}

/// Browse papers without search (PostgreSQL)
async fn browse_papers_postgres(
    state: &AppState,
    limit: usize,
    offset: usize,
    order: &str,
) -> Result<Json<search::SearchResponse<Paper>>, (StatusCode, Json<ApiError>)> {
    let papers: Vec<Paper> = sqlx::query_as(&format!(
        r#"
        SELECT id, title, abstract, arxiv_id, arxiv_url, pdf_url,
               published_date, authors, created_at, updated_at
        FROM papers
        ORDER BY published_date {} NULLS LAST
        LIMIT $1 OFFSET $2
        "#,
        order
    ))
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })?;

    let total = papers.len();
    Ok(Json(search::SearchResponse {
        papers,
        total_hits: total,
        facets: None,
    }))
}

async fn get_paper_by_id(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<PaperWithImplementations>, (StatusCode, Json<ApiError>)> {
    let paper = sqlx::query_as::<_, Paper>(
        r#"
        SELECT id, title, abstract, arxiv_id, arxiv_url, pdf_url,
               published_date, authors, created_at, updated_at
        FROM papers WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })?;

    let paper = paper.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Paper not found".to_string(),
            }),
        )
    })?;

    let implementations = sqlx::query_as::<_, Implementation>(
        r#"
        SELECT id, paper_id, github_url, framework, stars, is_official, created_at, updated_at
        FROM implementations WHERE paper_id = $1
        "#,
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Ok(Json(PaperWithImplementations {
        paper,
        implementations,
    }))
}

// ============================================================================
// Handlers: Datasets
// ============================================================================

async fn get_datasets(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Dataset>>, (StatusCode, Json<ApiError>)> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let datasets = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        sqlx::query_as::<_, Dataset>(
            r#"
            SELECT id, name, description, modalities, task_categories, languages,
                   size, homepage_url, github_url, paper_url, created_at, updated_at
            FROM datasets
            WHERE name ILIKE $1 OR description ILIKE $1
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as::<_, Dataset>(
            r#"
            SELECT id, name, description, modalities, task_categories, languages,
                   size, homepage_url, github_url, paper_url, created_at, updated_at
            FROM datasets
            ORDER BY name
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    };

    datasets.map(Json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })
}

async fn get_dataset_by_id(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Dataset>, (StatusCode, Json<ApiError>)> {
    let dataset = sqlx::query_as::<_, Dataset>(
        r#"
        SELECT id, name, description, modalities, task_categories, languages,
               size, homepage_url, github_url, paper_url, created_at, updated_at
        FROM datasets WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })?;

    dataset.map(Json).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Dataset not found".to_string(),
            }),
        )
    })
}

// ============================================================================
// Handlers: Benchmarks
// ============================================================================

async fn get_benchmarks(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Benchmark>>, (StatusCode, Json<ApiError>)> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let benchmarks = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        sqlx::query_as::<_, Benchmark>(
            r#"
            SELECT id, name, dataset_id, task, description, created_at, updated_at
            FROM benchmarks
            WHERE name ILIKE $1 OR task ILIKE $1
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as::<_, Benchmark>(
            r#"
            SELECT id, name, dataset_id, task, description, created_at, updated_at
            FROM benchmarks
            ORDER BY name
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    };

    benchmarks.map(Json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })
}

async fn get_benchmark_by_id(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<BenchmarkWithDataset>, (StatusCode, Json<ApiError>)> {
    let benchmark = sqlx::query_as::<_, Benchmark>(
        r#"
        SELECT id, name, dataset_id, task, description, created_at, updated_at
        FROM benchmarks WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })?;

    let benchmark = benchmark.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Benchmark not found".to_string(),
            }),
        )
    })?;

    let dataset = if let Some(dataset_id) = benchmark.dataset_id {
        sqlx::query_as::<_, Dataset>(
            r#"
            SELECT id, name, description, modalities, task_categories, languages,
                   size, homepage_url, github_url, paper_url, created_at, updated_at
            FROM datasets WHERE id = $1
            "#,
        )
        .bind(dataset_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
    } else {
        None
    };

    Ok(Json(BenchmarkWithDataset { benchmark, dataset }))
}

// ============================================================================
// Handlers: Implementations
// ============================================================================

async fn get_implementations(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Implementation>>, (StatusCode, Json<ApiError>)> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let implementations = sqlx::query_as::<_, Implementation>(
        r#"
        SELECT id, paper_id, github_url, framework, stars, is_official, created_at, updated_at
        FROM implementations
        ORDER BY stars DESC NULLS LAST
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await;

    implementations.map(Json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })
}

async fn get_implementation_by_id(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Implementation>, (StatusCode, Json<ApiError>)> {
    let implementation = sqlx::query_as::<_, Implementation>(
        r#"
        SELECT id, paper_id, github_url, framework, stars, is_official, created_at, updated_at
        FROM implementations WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })?;

    implementation.map(Json).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Implementation not found".to_string(),
            }),
        )
    })
}

// ============================================================================
// Handlers: Benchmark Results
// ============================================================================

async fn get_benchmark_results(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<BenchmarkResult>>, (StatusCode, Json<ApiError>)> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let results = sqlx::query_as::<_, BenchmarkResult>(
        r#"
        SELECT id, paper_id, benchmark_id, implementation_id, metric_name,
               metric_value, extra_data, created_at
        FROM benchmark_results
        ORDER BY metric_value DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await;

    results.map(Json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
    })
}
