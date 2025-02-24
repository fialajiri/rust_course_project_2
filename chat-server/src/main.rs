use chat_common::{Args, Message, MessageStream};
use clap::Parser;
use std::{
    io,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

type Clients = Arc<Mutex<Vec<TcpStream>>>;

fn main() {
    let args = Args::parse();
    let listener = TcpListener::bind(args.addr()).expect("Failed to bind to address");
    println!("Server listening on {}", args.addr());

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_new_client(stream, &clients),
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

fn handle_new_client(stream: TcpStream, clients: &Clients) {
    let addr = stream.peer_addr().unwrap();
    let clients = Arc::clone(clients);

    clients.lock().unwrap().push(stream.try_clone().unwrap());

    println!("New client connected: {}", addr);

    thread::spawn(move || {
        if let Err(e) = handle_connection(stream, &clients) {
            eprintln!("Error handling connection from {}: {}", addr, e);
        }
    });
}

fn handle_connection(mut stream: TcpStream, clients: &Clients) -> io::Result<()> {
    loop {
        match stream.read_message() {
            Ok(message) => {
                if let Err(e) = process_message(&stream, &message, clients) {
                    eprintln!("Error processing message: {}", e);
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}

fn process_message(stream: &TcpStream, message: &Message, clients: &Clients) -> io::Result<()> {
    // Log message
    match message {
        Message::Text(text) => println!("{}: {}", stream.peer_addr()?, text),
        Message::File { name, .. } => println!("{}: Sent file {}", stream.peer_addr()?, name),
        Message::Image { name, .. } => println!("{}: Sent image {}", stream.peer_addr()?, name),
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
