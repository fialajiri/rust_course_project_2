use chat_common::{file_ops, Args, Message, MessageStream};
use clap::Parser;
use std::{
    fs,
    io::{self, BufRead},
    net::TcpStream,
    thread,
};
use tracing::{error, info, warn};

fn main() -> io::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let stream = TcpStream::connect(args.addr())?;
    let receiver_stream = stream.try_clone()?;
    info!("Connected to {}", args.addr());

    // Create directories if they don't exist
    fs::create_dir_all("images")?;
    fs::create_dir_all("files")?;

    spawn_receiver_thread(receiver_stream);

    handle_outgoing_messages(stream)
}

fn spawn_receiver_thread(mut stream: TcpStream) {
    thread::spawn(move || {
        if let Err(e) = handle_incoming(&mut stream) {
            error!("Error handling incoming messages: {}", e);
        }
    });
}

fn handle_outgoing_messages(mut stream: TcpStream) -> io::Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        if line == ".quit" {
            break;
        }

        let message = parse_and_process_message(&line)?;
        if let Some(msg) = message {
            stream.write_message(&msg)?;
        }
    }

    Ok(())
}

fn parse_and_process_message(line: &str) -> io::Result<Option<Message>> {
    if !line.starts_with(".file ") && !line.starts_with(".image ") {
        return Ok(Some(Message::Text(line.to_string())));
    }

    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() != 2 {
        warn!("Invalid command format. Use: .file <path> or .image <path>");
        return Ok(None);
    }

    let command = parts[0];
    let path = parts[1];

    match file_ops::process_file_command(command, path) {
        Ok(msg) => Ok(Some(msg)),
        Err(e) => {
            error!("Error processing file: {}", e);
            Ok(None)
        }
    }
}

fn handle_incoming(stream: &mut TcpStream) -> io::Result<()> {
    while let Ok(message) = stream.read_message() {
        match message {
            Message::Text(text) => {
                info!("Received: {}", text);
            }
            Message::File { name, data } => {
                info!("Receiving file: {}", name);
                file_ops::save_file(&name, data)?;
            }
            Message::Image { name, data } => {
                info!("Receiving image: {}", name);
                file_ops::save_image(&name, data)?;
            }
        }
    }
    Ok(())
}
