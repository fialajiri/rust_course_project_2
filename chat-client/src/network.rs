use std::net::TcpStream;
use std::thread;
use tracing::error;

use crate::message_handler::handle_incoming;

pub fn spawn_receiver_thread(mut stream: TcpStream) {
    thread::spawn(move || {
        if let Err(e) = handle_incoming(&mut stream) {
            error!("Error handling incoming messages: {}", e);
        }
    });
}
