use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenvy::dotenv;
use backend::{create_app, search::SearchIndex};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("POSTGRES_URI")
        .expect("POSTGRES_URI must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Try to load Tantivy search index (optional)
    let index_path = env::var("TANTIVY_INDEX_PATH")
        .unwrap_or_else(|_| "./data/tantivy_index".to_string());

    let search_index = match SearchIndex::open(&index_path) {
        Ok(index) => {
            println!("Tantivy search index loaded from {}", index_path);
            Some(Arc::new(index))
        }
        Err(e) => {
            println!(
                "Tantivy search index not available at {} ({}). Using PostgreSQL fallback.",
                index_path, e
            );
            println!("Run `cargo run --bin build_search_index` to build the index.");
            None
        }
    };

    let app = create_app(pool, search_index);

    // Run our application
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
