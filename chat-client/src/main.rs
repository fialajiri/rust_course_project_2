mod commands;
mod message_handler;
mod network;
mod ui;

use anyhow::{Context, Result};
use chat_common::Args;
use clap::Parser;
use std::fs;
use tokio::net::TcpStream;
use tracing::info;

use network::spawn_receiver_task;

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

    // Create directories if they don't exist
    fs::create_dir_all("images").context("Failed to create images directory")?;
    fs::create_dir_all("files").context("Failed to create files directory")?;

    spawn_receiver_task(receiver_stream);

    ui::run_input_loop(writer_stream).await
}
