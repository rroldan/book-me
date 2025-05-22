// Import necessary crates and modules
use axum::{
    extract::State, http::StatusCode, routing::{get, post}, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Pool, Database, Sqlite, SqlitePool, PgPool, AnyKind};
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

// Generic function to create the database schema.
async fn create_schema<DB: Database>(pool: &Pool<DB>, db_type: &str) -> Result<(), sqlx::Error> {
    let schema_sql = if db_type == "postgres" {
        "CREATE TABLE IF NOT EXISTS messages (id SERIAL PRIMARY KEY, text TEXT NOT NULL);"
    } else { // sqlite
        "CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY AUTOINCREMENT, text TEXT NOT NULL);"
    };
    sqlx::query(schema_sql).execute(pool).await?;
    println!("'messages' table checked/created successfully for {} using schema: {}", db_type, schema_sql);
    Ok(())
}

// Generic function to run the application.
pub async fn run_app<DB: Database + 'static>(
    pool: Pool<DB>,
    listener: tokio::net::TcpListener,
    db_type: &'static str, // Pass db_type for schema creation and potentially other logic
) -> Result<()>
where
    for<'a> &'a Pool<DB>: sqlx::Executor<'a, Database = DB>,
{
    // Schema creation
    create_schema(&pool, db_type).await.expect("Failed to create schema");

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/json", get(json_handler)) // json_handler is simple, no DB interaction
        .route("/messages", post(create_message_handler::<DB>))
        .route("/messages", get(list_messages_handler::<DB>))
        .with_state(pool); // Pass the generic pool

    println!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

// Function to setup and run the application with SQLite.
pub async fn run_app_sqlite() -> Result<()> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for main application");
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        println!("Creating database {}", db_url);
        Sqlite::create_database(&db_url).await?;
    } else {
        println!("Database {} already exists.", db_url);
    }
    let pool = SqlitePool::connect(&db_url).await?;
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    run_app(pool, listener, "sqlite").await
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

// Generic handler for creating a new message (POST `/messages`).
async fn create_message_handler<DB: Database>(
    State(pool): State<Pool<DB>>,      // Extract the database pool from application state.
    Json(payload): Json<CreateMessage>, // Deserialize the request body into `CreateMessage`.
) -> Result<StatusCode, (StatusCode, String)>
where
    for<'c> &'c Pool<DB>: sqlx::Executor<'c, Database = DB>,
{
    // Determine placeholder style based on database kind
    let query_str = match pool.any_kind() {
        AnyKind::Postgres => "INSERT INTO messages (text) VALUES ($1)",
        _ => "INSERT INTO messages (text) VALUES (?1)", // Default to SQLite style
    };

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

// Generic handler for listing all messages (GET `/messages`).
async fn list_messages_handler<DB: Database>(
    State(pool): State<Pool<DB>>, // Extract the database pool from application state.
) -> Result<Json<Vec<Message>>, (StatusCode, String)>
where
    for<'c> &'c Pool<DB>: sqlx::Executor<'c, Database = DB>,
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
