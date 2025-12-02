use axum::{
    extract::State,
    routing::get,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use sqlx::{Pool, Postgres};

#[derive(Serialize)]
pub struct Message {
    pub message: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug, PartialEq)]
pub struct Paper {
    pub id: uuid::Uuid,
    pub title: String,
    pub published_date: Option<chrono::NaiveDate>,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
}

pub fn create_app(pool: Pool<Postgres>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState { pool };

    Router::new()
        .route("/", get(root))
        .route("/api/health", get(health_check))
        .route("/api/papers", get(get_papers))
        .layer(cors)
        .with_state(state)
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn health_check() -> Json<Message> {
    Json(Message {
        message: "Backend is running!".to_string(),
    })
}

async fn get_papers(State(state): State<AppState>) -> Json<Vec<Paper>> {
    let papers = sqlx::query_as::<_, Paper>("SELECT id, title, published_date FROM papers LIMIT 10")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_else(|_| vec![]);

    Json(papers)
}
