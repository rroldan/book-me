// axum_app/tests/integration_tests.rs
#![allow(dead_code)] // For items in lib.rs that might not be used by main binary

use axum_app::{run_app, Message, CreateMessage}; // Assumes axum_app is the crate name
use reqwest;
use sqlx::PgPool;
use std::net::TcpListener as StdTcpListener;
use testcontainers::{clients, images, Docker};
use testcontainers_modules::postgres::Postgres as PostgresContainer; // Alias to avoid conflict
use tokio::net::TcpListener as TokioTcpListener;

fn get_free_port() -> u16 {
    StdTcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

#[tokio::test]
async fn test_create_and_list_messages() {
    // 1. Setup Testcontainers
    let docker = clients::Cli::default();
    // Explicitly use testcontainers_modules::images::postgres::Postgres here if just images::postgres is ambiguous
    let postgres_image = images::postgres::Postgres::default().with_version(15);
    let node = docker.run(postgres_image);
    let port = node.get_host_port_ipv4(5432);

    // 2. Setup Database Connection
    let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to test Postgres database");

    // 3. Setup and run the Axum application
    let free_port = get_free_port();
    let listener = TokioTcpListener::bind(format!("127.0.0.1:{}", free_port))
        .await
        .expect("Failed to bind to a test port");
    let app_url = format!("http://127.0.0.1:{}", free_port);

    let test_pool = pool.clone(); // Clone pool for the app task
    tokio::spawn(async move {
        run_app(test_pool, listener, "postgres")
            .await
            .expect("Test application failed to run");
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 4. Perform HTTP requests
    let client = reqwest::Client::new();
    let create_payload = CreateMessage {
        text: "Hello from integration test!".to_string(),
    };
    let post_response = client
        .post(format!("{}/messages", app_url))
        .json(&create_payload)
        .send()
        .await
        .expect("Failed to send POST /messages request");

    assert_eq!(post_response.status(), reqwest::StatusCode::CREATED);

    let get_response = client
        .get(format!("{}/messages", app_url))
        .send()
        .await
        .expect("Failed to send GET /messages request");

    assert_eq!(get_response.status(), reqwest::StatusCode::OK);

    let messages: Vec<Message> = get_response
        .json()
        .await
        .expect("Failed to parse JSON response from GET /messages");

    assert_eq!(messages.len(), 1, "Expected one message after creation.");
    assert_eq!(messages[0].text, "Hello from integration test!");
    assert_eq!(messages[0].id, 1, "Expected ID of the first message to be 1.");
}
