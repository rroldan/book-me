// axum_app/src/main.rs
use axum_app::run_app_sqlite;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    run_app_sqlite().await
}
