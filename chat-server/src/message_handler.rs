use crate::types::Clients;
use anyhow::Result;
use chat_common::{Message, MessageStream};
use std::net::TcpStream;
use tracing::{error, info};

#[derive(Clone)]
pub struct MessageHandler {
    clients: Clients,
}

impl MessageHandler {
    pub fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub fn process_message(&self, stream: &TcpStream, message: &Message) -> Result<()> {
        if let Message::Error {
            code,
            message: error_msg,
        } = message
        {
            info!("{}: Error {:?}: {}", stream.peer_addr()?, code, error_msg);
            return Ok(());
        }

        match message {
            Message::Text(text) => info!("{}: {}", stream.peer_addr()?, text),
            Message::File { name, .. } => info!("{}: Sent file {}", stream.peer_addr()?, name),
            Message::Image { name, .. } => info!("{}: Sent image {}", stream.peer_addr()?, name),
            _ => {}
        }

        let ack_message = match message {
            Message::Text(_) => Message::System("Message sent successfully".to_string()),
            Message::File { name, .. } => {
                Message::System(format!("File '{}' sent successfully", name))
            }
            Message::Image { name, .. } => {
                Message::System(format!("Image '{}' sent successfully", name))
            }
            _ => return self.broadcast_message(stream, message),
        };

        if let Ok(mut sender_stream) = stream.try_clone() {
            if let Err(e) = sender_stream.write_message(&ack_message) {
                error!("Failed to send acknowledgment: {}", e);
            }
        }

        self.broadcast_message(stream, message)
    }

    pub fn handle_disconnect(&self, stream: &TcpStream) -> Result<()> {
        let addr = stream.peer_addr()?;
        let mut clients_lock = self.clients.lock().unwrap();

        clients_lock.retain(|client| {
            client
                .peer_addr()
                .map(|client_addr| client_addr != addr)
                .unwrap_or(false)
        });

        let disconnect_msg = Message::System(format!("Client {} has disconnected", addr));

        for client in clients_lock.iter() {
            if let Ok(mut c) = client.try_clone() {
                let _ = c.write_message(&disconnect_msg);
            }
        }

        info!("Client disconnected: {}", addr);
        Ok(())
    }

    fn broadcast_message(&self, sender: &TcpStream, message: &Message) -> Result<()> {
        let sender_addr = sender.peer_addr()?;
        let mut clients = self.clients.lock().unwrap();
        let mut failed_clients = Vec::new();

        for (index, client) in clients.iter().enumerate() {
            if client.peer_addr().unwrap() != sender_addr {
                let result = client
                    .try_clone()
                    .map(|mut c| c.write_message(message).is_ok())
                    .unwrap_or(false);

                if !result {
                    failed_clients.push(index);
                }
            }
        }

        for index in failed_clients.into_iter().rev() {
            let removed_client = clients.remove(index);
            if let Ok(addr) = removed_client.peer_addr() {
                error!("Removed disconnected client: {}", addr);
                let disconnect_msg = Message::Text(format!("Client {} disconnected", addr));

                for client in clients.iter() {
                    if let Ok(mut c) = client.try_clone() {
                        let _ = c.write_message(&disconnect_msg);
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::TcpListener,
        sync::{Arc, Mutex},
        thread,
    };

    #[test]
    fn test_process_text_message() {
        let clients: Clients = Arc::new(Mutex::new(Vec::new()));
        let handler = MessageHandler::new(clients);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let mut client = TcpStream::connect(addr).unwrap();
            let test_message = Message::Text("Test message".to_string());
            client.write_message(&test_message).unwrap();
        });

        if let Ok((stream, _)) = listener.accept() {
            let message = Message::Text("Test message".to_string());
            handler.process_message(&stream, &message).unwrap();
        }
    }

    #[test]
    fn test_handle_disconnect() {
        let clients: Clients = Arc::new(Mutex::new(Vec::new()));
        let handler = MessageHandler::new(clients.clone());

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let _client = TcpStream::connect(addr).unwrap();
        let (server_stream, _) = listener.accept().unwrap();

        clients
            .lock()
            .unwrap()
            .push(server_stream.try_clone().unwrap());

        handler.handle_disconnect(&server_stream).unwrap();

        assert_eq!(clients.lock().unwrap().len(), 0);
    }
}
