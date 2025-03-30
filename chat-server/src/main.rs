use anyhow::{Context, Result as AnyhowResult};
use chat_common::error::ChatError;
use chat_server::routes::messages;
use chat_server::routes::users;
use chat_server::services::client_service::ClientService;
use chat_server::utils::cors::Cors;
use chat_server::utils::db_connection::{self, DbConn};
use rocket_db_pools::Database;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

const DEFAULT_ADDRESS: &str = "0.0.0.0";
const DEFAULT_TCP_PORT: &str = "8080";

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    tracing_subscriber::fmt::init();

    // Initialize database pool for the TCP server
    let pool = db_connection::create_pool().await?;
    let pool = Arc::new(pool);
    info!("Database connection pool established");

    // Set up the TCP server
    let addr = env::var("SERVER_ADDRESS").unwrap_or_else(|_| DEFAULT_ADDRESS.to_string());
    let tcp_port = env::var("TCP_PORT").unwrap_or_else(|_| DEFAULT_TCP_PORT.to_string());
    let tcp_addr = format!("{}:{}", addr, tcp_port);
    let listener = tokio::net::TcpListener::bind(&tcp_addr)
        .await
        .context("Failed to bind to TCP address")?;

    info!("TCP Server listening on {}", tcp_addr);

    // Initialize client handler
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let client_handler = ClientService::new(clients, pool.clone())?;

    // Start Rocket server in a separate task
    tokio::spawn(async move {
        let _rocket = rocket::build()
            .attach(DbConn::init())
            .attach(Cors)
            .mount("/users", users::routes())
            .mount("/messages", messages::routes())
            .launch()
            .await
            .expect("Failed to launch Rocket server");
    });

    // Main server loop
    info!("Server started and ready to accept connections");
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New TCP connection from: {}", addr);
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
