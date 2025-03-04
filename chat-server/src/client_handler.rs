use crate::types::Clients;
use chat_common::error::{ChatError, Result};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use tracing::{error, info};

pub struct ClientHandler {
    clients: Clients,
}

impl ClientHandler {
    pub fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub fn handle_new_client(&self, stream: TcpStream) -> Result<()> {
        let addr = stream
            .peer_addr()
            .map_err(|e| ChatError::NetworkError(format!("Failed to get peer address: {}", e)))?;

        let clients = Arc::clone(&self.clients);

        let cloned_stream = stream
            .try_clone()
            .map_err(|e| ChatError::NetworkError(format!("Failed to clone stream: {}", e)))?;

        let mut clients_guard = clients.lock().map_err(|e| {
            ChatError::ServerError(format!("Failed to acquire clients lock: {}", e))
        })?;

        clients_guard.push(cloned_stream);
        drop(clients_guard);

        info!("New client connected: {}", addr);

        let connection_handler = crate::ConnectionHandler::new(Arc::clone(&clients));
        thread::spawn(move || {
            if let Err(e) = connection_handler.handle_connection(stream) {
                error!("Error handling connection from {}: {}", addr, e);
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::sync::{mpsc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_handle_new_client() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let clients: Clients = Arc::new(Mutex::new(Vec::new()));
        let client_handler = ClientHandler::new(clients.clone());

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(b"Hello, server!").unwrap();
            tx.send(()).unwrap();
        });

        if let Ok((stream, _)) = listener.accept() {
            client_handler.handle_new_client(stream).unwrap();
        }

        rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(clients.lock().unwrap().len(), 1);
    }
}
