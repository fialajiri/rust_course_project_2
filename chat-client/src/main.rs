mod commands;
mod message_handler;
mod network;
mod ui;

use anyhow::{Context, Result};
use chat_common::{encryption::EncryptionService, Args};
use clap::Parser;
use std::{fs, sync::Arc};
use tokio::net::TcpStream;
use tracing::info;

use network::spawn_receiver_task;

const ENCRYPTION_KEY: [u8; 32] = [0; 32];

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    println!("Connecting to {}", args.addr());
    let stream = TcpStream::connect(args.addr())
        .await
        .context("Failed to connect to server")?;
    let (receiver_stream, writer_stream) = stream.into_split();
    info!("Connected to {}", args.addr());

    // Initialize encryption service
    let encryption = Arc::new(EncryptionService::new(&ENCRYPTION_KEY)?);

    // Create directories if they don't exist
    fs::create_dir_all("images").context("Failed to create images directory")?;
    fs::create_dir_all("files").context("Failed to create files directory")?;

    spawn_receiver_task(receiver_stream, Arc::clone(&encryption));

    ui::run_input_loop(writer_stream, Arc::clone(&encryption)).await
}
