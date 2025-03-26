mod commands;
mod message_handler;
mod network;
mod ui;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chat_common::{encryption::EncryptionService, Args};
use clap::Parser;
use std::{fs, sync::Arc};
use tokio::net::TcpStream;
use tracing::{info, warn};

use network::spawn_receiver_task;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing first
    tracing_subscriber::fmt::init();

    match dotenvy::dotenv() {
        Ok(_) => info!("Successfully loaded .env file"),
        Err(e) => warn!("Failed to load .env file: {}", e),
    }

    let args = Args::parse();
    println!("Connecting to {}", args.addr());
    let stream = TcpStream::connect(args.addr())
        .await
        .context("Failed to connect to server")?;
    let (receiver_stream, writer_stream) = stream.into_split();
    info!("Connected to {}", args.addr());

    // Initialize encryption service
    let key =
        std::env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY environment variable must be set");

    let key_bytes = BASE64
        .decode(key)
        .expect("ENCRYPTION_KEY must be valid base64");

    if key_bytes.len() != 32 {
        panic!("ENCRYPTION_KEY must be exactly 32 bytes when decoded");
    }

    let encryption = Arc::new(EncryptionService::new(&key_bytes)?);

    // Create directories if they don't exist
    fs::create_dir_all("images").context("Failed to create images directory")?;
    fs::create_dir_all("files").context("Failed to create files directory")?;

    spawn_receiver_task(receiver_stream, Arc::clone(&encryption));

    ui::run_input_loop(writer_stream, Arc::clone(&encryption)).await
}
