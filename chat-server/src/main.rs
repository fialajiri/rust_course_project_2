use anyhow::{Context, Result as AnyhowResult};
use chat_common::error::ChatError;
use chat_common::Args;
use chat_server::ClientHandler;
use clap::Parser;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

fn main() -> AnyhowResult<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let listener = TcpListener::bind(args.addr()).context("Failed to bind to address")?;
    info!("Server listening on {}", args.addr());

    let clients = Arc::new(Mutex::new(Vec::new()));
    let client_handler = ClientHandler::new(clients);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = client_handler.handle_new_client(stream) {
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

    Ok(())
}
