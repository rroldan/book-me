// Import necessary crates and modules
use axum::{
    extract::State, http::StatusCode, routing::{get, post}, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Pool, Database, PgPool, AnyKind};
use std::env;
use std::net::SocketAddr;
use anyhow::Result;

// Struct for deserializing the request body when creating a new message.
// It expects a JSON object with a "text" field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub text: String,
}

// Struct representing a message retrieved from the database.
// It includes an `id` and the `text` of the message.
// Derives `Serialize` for sending as JSON, `Clone` for potential copying,
// and `sqlx::FromRow` for mapping database rows to this struct.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i64,
    pub text: String,
}

// Function to create the database schema for PostgreSQL.
async fn create_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    let schema_sql = "CREATE TABLE IF NOT EXISTS messages (id SERIAL PRIMARY KEY, text TEXT NOT NULL);";
    sqlx::query(schema_sql).execute(pool).await?;
    println!("'messages' table checked/created successfully for PostgreSQL using schema: {}", schema_sql);
    Ok(())
}

// Function to run the application with PostgreSQL.
pub async fn run_app(
    pool: PgPool,
    listener: tokio::net::TcpListener,
) -> Result<()>
{
    // Schema creation
    create_schema(&pool).await.expect("Failed to create schema");

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/json", get(json_handler)) // json_handler is simple, no DB interaction
        .route("/messages", post(create_message_handler::<sqlx::Postgres>))
        .route("/messages", get(list_messages_handler::<sqlx::Postgres>))
        .with_state(pool); // Pass the PgPool

    println!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

// Handler for the root path (`/`).
// Returns a simple "Hello, World!" string.
async fn root_handler() -> &'static str {
    "Hello, World!"
}

// Handler for the `/json` path.
// Returns a sample JSON response using the `Message` struct.
async fn json_handler() -> Json<Message> {
    Json(Message {
        id: 0, // Dummy ID for this example.
        text: "Hello, JSON!".to_string(),
    })
}

// Handler for creating a new message (POST `/messages`) for PostgreSQL.
async fn create_message_handler<DB: Database>( // Still generic for now, but used with PgPool
    State(pool): State<Pool<DB>>,      // Extract the database pool from application state.
    Json(payload): Json<CreateMessage>, // Deserialize the request body into `CreateMessage`.
) -> Result<StatusCode, (StatusCode, String)>
where
    for<'c> &'c Pool<DB>: sqlx::Executor<'c, Database = DB>, // This bound is fine
{
    let query_str = "INSERT INTO messages (text) VALUES ($1)"; // PostgreSQL specific query

    sqlx::query(query_str)
        .bind(payload.text)
        .execute(&pool) // Use the connection pool.
        .await // Await the asynchronous operation.
        .map_err(|e| {
            // Handle potential errors during database insertion.
            eprintln!("Failed to create message: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create message".to_string())
        })?;
    // Return HTTP 201 Created status on success.
    Ok(StatusCode::CREATED)
}

// Handler for listing all messages (GET `/messages`) for PostgreSQL.
async fn list_messages_handler<DB: Database>( // Still generic for now, but used with PgPool
    State(pool): State<Pool<DB>>, // Extract the database pool from application state.
) -> Result<Json<Vec<Message>>, (StatusCode, String)>
where
    for<'c> &'c Pool<DB>: sqlx::Executor<'c, Database = DB>, // This bound is fine
{
    // The SQL query "SELECT id, text FROM messages ORDER BY id" is compatible with both SQLite and Postgres.
    // Using sqlx::query_as (the function, not the macro) for better generic type handling.
    let messages = sqlx::query_as::<_, Message>("SELECT id, text FROM messages ORDER BY id")
        .fetch_all(&pool) // Use the connection pool.
        .await // Await the asynchronous operation.
        .map_err(|e| {
            // Handle potential errors during database fetching.
            eprintln!("Failed to fetch messages: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch messages".to_string())
        })?;
    // Return the list of messages as JSON.
    Ok(Json(messages))
}
