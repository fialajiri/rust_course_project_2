mod commands;
mod message_handler;
mod network;
mod ui;

use anyhow::{Context, Result};
use chat_common::Args;
use clap::Parser;
use std::{fs, net::TcpStream};
use tracing::info;

use network::spawn_receiver_thread;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let stream = TcpStream::connect(args.addr()).context("Failed to connect to server")?;
    let receiver_stream = stream.try_clone().context("Failed to clone TCP stream")?;
    info!("Connected to {}", args.addr());

    // Create directories if they don't exist
    fs::create_dir_all("images").context("Failed to create images directory")?;
    fs::create_dir_all("files").context("Failed to create files directory")?;

    spawn_receiver_thread(receiver_stream);

    ui::run_input_loop(stream)
}
