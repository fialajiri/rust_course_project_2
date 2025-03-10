use anyhow::{Context, Result as AnyhowResult};
use chat_common::error::ChatError;
use chat_common::Args;
use chat_server::{db_connection, ClientHandler};
use clap::Parser;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    tracing_subscriber::fmt::init();

    // Initialize database connection
    let _db_connection = &mut db_connection::establish_connection();
    info!("Database connection established");

    // Set up the server
    let _args = Args::parse();

    // Explicitly bind to 0.0.0.0 to accept connections from outside the container
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;
    info!("Server listening on {}", addr);

    // Initialize client handler
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let client_handler = ClientHandler::new(clients);

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
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                // Don't break the loop on connection errors
            }
        }
    }
}
