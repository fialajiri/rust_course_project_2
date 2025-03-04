use crate::types::Clients;
use crate::MessageHandler;
use anyhow::Result;
use chat_common::{Message, MessageStream};
use std::net::TcpStream;
use tracing::error;

pub struct ConnectionHandler {
    clients: Clients,
}

impl ConnectionHandler {
    pub fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let addr = stream.peer_addr()?;
        let message_handler = MessageHandler::new(self.clients.clone());

        let mut message_result = stream.read_message();
        while let Ok(message) = message_result {
            if let Err(e) = message_handler.process_message(&stream, &message) {
                error!("Error processing message from {}: {}", addr, e);

                let error_message = Message::Error {
                    code: chat_common::ErrorCode::ServerError,
                    message: format!("Server error: {}", e),
                };

                if let Err(send_err) = stream.write_message(&error_message) {
                    error!("Failed to send error message to client: {}", send_err);
                    break;
                }
            }

            message_result = stream.read_message();
        }

        message_handler.handle_disconnect(&stream)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_handle_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let clients: Clients = Arc::new(Mutex::new(Vec::new()));

        let handler = ConnectionHandler::new(clients.clone());

        thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                handler.handle_connection(stream).unwrap();
            }
        });

        let mut client = TcpStream::connect(addr).unwrap();
        let test_message = Message::Text("Hello".to_string());
        client.write_message(&test_message).unwrap();

        // Read acknowledgment
        let response = client.read_message().unwrap();
        match response {
            Message::System(msg) => assert!(msg.contains("successfully")),
            _ => panic!("Expected system message"),
        }
    }
}
