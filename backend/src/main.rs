use std::net::SocketAddr;
use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenvy::dotenv;
use backend::create_app;

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

    let app = create_app(pool);

    // Run our application
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
