use anyhow::{Context, Result};
use chat_common::{Args, Message, MessageStream};
use clap::Parser;
use std::{
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};
use tracing::{error, info};

type Clients = Arc<Mutex<Vec<TcpStream>>>;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let listener = TcpListener::bind(args.addr()).context("Failed to bind to address")?;
    info!("Server listening on {}", args.addr());

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_new_client(stream, &clients),
            Err(e) => error!("Connection failed: {}", e),
        }
    }

    Ok(())
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

fn handle_connection(mut stream: TcpStream, clients: &Clients) -> Result<()> {
    while let Ok(message) = stream.read_message() {
        if let Err(e) = process_message(&stream, &message, clients) {
            error!("Error processing message: {}", e);
            break;
        }
    }
    Ok(())
}

fn process_message(stream: &TcpStream, message: &Message, clients: &Clients) -> Result<()> {
    match message {
        Message::Text(text) => info!("{}: {}", stream.peer_addr()?, text),
        Message::File { name, .. } => info!("{}: Sent file {}", stream.peer_addr()?, name),
        Message::Image { name, .. } => info!("{}: Sent image {}", stream.peer_addr()?, name),
    }

    broadcast_message(stream, message, clients)
}

fn broadcast_message(sender: &TcpStream, message: &Message, clients: &Clients) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_handle_new_client() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let clients: Clients = Arc::new(Mutex::new(Vec::new()));

        let (tx, rx) = mpsc::channel();

        // Spawn a thread to simulate a client connection
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(b"Hello, server!").unwrap();
            tx.send(()).unwrap(); // Notify that the client has connected
        });

        // Accept the connection and handle the new client
        if let Ok((stream, _)) = listener.accept() {
            handle_new_client(stream, &clients);
        }

        // Wait for the client to connect
        rx.recv_timeout(Duration::from_secs(1)).unwrap();

        // Check if the client was added to the clients list
        assert_eq!(clients.lock().unwrap().len(), 1);
    }
}
