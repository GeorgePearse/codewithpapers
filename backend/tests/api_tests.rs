use backend::create_app;
use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenvy::dotenv;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn health_check_works() {
    dotenv().ok();
    let database_url = env::var("POSTGRES_URI").expect("POSTGRES_URI must be set");
    
    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn can_fetch_papers() {
    dotenv().ok();
    let database_url = env::var("POSTGRES_URI").expect("POSTGRES_URI must be set");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Check if tables exist by trying to query
    let row: (i64,) = sqlx::query_as("SELECT count(*) FROM papers")
        .fetch_one(&pool)
        .await
        .expect("Failed to query papers table. Does it exist?");

    println!("Found {} papers", row.0);

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/papers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    // We could parse the body here if we wanted to be sure it returns JSON
}
