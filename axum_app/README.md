# Axum App

A simple Rust web application built with Axum, Tokio, SQLx (SQLite), and Serde.

## Prerequisites

- Rust programming language: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- (Optional, for `sqlx-cli` if you want to manage migrations later): `cargo install sqlx-cli`

## Setup

1.  **Clone the repository (or ensure you are in the `axum_app` directory).**
2.  **Database Setup:**
    - The application uses SQLite. A `.env` file is used to configure the database URL.
    - Create a `.env` file in the `axum_app` directory (if it doesn't exist):
      ```
      DATABASE_URL=sqlite:database.db
      ```
    - The database file (`database.db`) will be created automatically in the `axum_app` directory when the application starts if it doesn't already exist. The `messages` table will also be created automatically.

## Running the Application

1.  Navigate to the `axum_app` directory.
2.  Run the application:
    ```bash
    cargo run
    ```
3.  The application will start, and you should see a message like:
    `Listening on http://0.0.0.0:3000`

## API Endpoints

-   `GET /`: Returns a "Hello, World!" plain text message.
-   `GET /json`: Returns a sample JSON message: `{"id":0,"text":"Hello, JSON!"}`.
-   `POST /messages`: Creates a new message.
    -   Request body (JSON): `{"text": "Your message content"}`
    -   Response: `201 CREATED` on success.
-   `GET /messages`: Lists all stored messages.
    -   Response: JSON array of messages, e.g., `[{"id":1,"text":"First message"},{"id":2,"text":"Another message"}]`.

## Project Structure

-   `src/main.rs`: Contains the main application logic, including route handlers, database interaction, and server setup.
-   `Cargo.toml`: Manages project dependencies.
-   `.env`: (Gitignored) Stores environment variables like `DATABASE_URL`.
-   `database.db`: (Gitignored) The SQLite database file.
-   `tests/integration_tests.rs`: Contains integration tests using Testcontainers.

## Running Integration Tests

Integration tests use Testcontainers to spin up a real Postgres database instance. Ensure Docker is installed and running on your system.

To run the integration tests, navigate to the `axum_app` directory and execute:

```bash
cargo test --test integration_tests -- --test-threads=1
```

The `--test-threads=1` flag is recommended for Testcontainers to ensure test isolation and prevent port conflicts or resource exhaustion.

## CI/CD

This project uses GitHub Actions for continuous integration and continuous delivery (CI/CD). The workflow is defined in `.github/workflows/rust.yml`.

Key features of the CI/CD pipeline:
-   **Builds the application**: Ensures the code compiles successfully.
-   **Runs all tests**: Executes unit tests and integration tests (which use Testcontainers with Docker).
-   **Triggered on events**: Runs automatically on pushes and pull requests to the `main` branch.
-   **Environment**: Uses Rust `1.82.0` and includes Docker for running Testcontainer-based tests.
