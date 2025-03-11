use anyhow::{Context, Result as AnyhowResult};
use chat_common::error::ChatError;
use chat_server::services::client_service::ClientService;
use chat_server::utils::db_connection;

use std::env;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

const DEFAULT_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: &str = "8080";

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    tracing_subscriber::fmt::init();

    // Initialize database pool
    let pool = db_connection::create_pool().await?;
    let pool = Arc::new(pool);
    info!("Database connection pool established");

    // Set up the server
    let addr = env::var("SERVER_ADDRESS").unwrap_or_else(|_| DEFAULT_ADDRESS.to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
    let addr = format!("{}:{}", addr, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    info!("Server listening on {}", addr);

    // Initialize client handler
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let client_handler = ClientService::new(clients, Arc::clone(&pool));

    // Main server loop
    info!("Server started and ready to accept connections");
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New connection from: {}", addr);
                if let Err(e) = client_handler.handle_new_client(stream).await {
                    error!(
                        "Failed to handle client: {} (code: {:?})",
                        e,
                        e.to_error_code()
                    );
                }
            }
            Err(e) => error!(
                "Connection failed: {}",
                ChatError::NetworkError(e.to_string())
            ),
        }
    }
}
