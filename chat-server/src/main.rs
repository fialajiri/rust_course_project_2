use chat_common::{Args, Message, MessageStream};
use clap::Parser;
use std::{
    io,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};
use tracing::{error, info};

type Clients = Arc<Mutex<Vec<TcpStream>>>;

fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let listener = TcpListener::bind(args.addr()).expect("Failed to bind to address");
    info!("Server listening on {}", args.addr());

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_new_client(stream, &clients),
            Err(e) => error!("Connection failed: {}", e),
        }
    }
}

fn handle_new_client(stream: TcpStream, clients: &Clients) {
    let addr = stream.peer_addr().unwrap();
    let clients = Arc::clone(clients);

    clients.lock().unwrap().push(stream.try_clone().unwrap());

    info!("New client connected: {}", addr);

    thread::spawn(move || {
        if let Err(e) = handle_connection(stream, &clients) {
            error!("Error handling connection from {}: {}", addr, e);
        }
    });
}

fn handle_connection(mut stream: TcpStream, clients: &Clients) -> io::Result<()> {
    while let Ok(message) = stream.read_message() {
        if let Err(e) = process_message(&stream, &message, clients) {
            error!("Error processing message: {}", e);
            break;
        }
    }
    Ok(())
}

fn process_message(stream: &TcpStream, message: &Message, clients: &Clients) -> io::Result<()> {
    // Log message
    match message {
        Message::Text(text) => info!("{}: {}", stream.peer_addr()?, text),
        Message::File { name, .. } => info!("{}: Sent file {}", stream.peer_addr()?, name),
        Message::Image { name, .. } => info!("{}: Sent image {}", stream.peer_addr()?, name),
    }

    // Broadcast message, excluding the sender
    broadcast_message(stream, message, clients)
}

fn broadcast_message(sender: &TcpStream, message: &Message, clients: &Clients) -> io::Result<()> {
    let sender_addr = sender.peer_addr()?;
    let mut clients = clients.lock().unwrap();
    clients.retain(|client| {
        if client.peer_addr().unwrap() != sender_addr {
            client
                .try_clone()
                .map(|mut c| c.write_message(message).is_ok())
                .unwrap_or(false)
        } else {
            true
        }
    });
    Ok(())
}
