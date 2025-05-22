// axum_app/src/main.rs
use axum_app::run_app; // Updated import
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // DATABASE_URL will now be for PostgreSQL
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set and point to a PostgreSQL instance");
    
    // Connect to PostgreSQL
    let pool = PgPool::connect(&database_url).await?;
    
    // Set up listener
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000)); // Or make port configurable
    let listener = TcpListener::bind(addr).await?;
    
    // Run the application
    println!("Starting server with PostgreSQL backend..."); // Optional: some indication
    run_app(pool, listener).await
}
