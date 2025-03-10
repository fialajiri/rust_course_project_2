use crate::types::{ChatRoomConnection, Clients};
use chat_common::error::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{error, info};

pub struct ClientHandler {
    clients: Clients,
    next_id: AtomicUsize,
}

impl ClientHandler {
    pub fn new(clients: Clients) -> Self {
        Self {
            clients,
            next_id: AtomicUsize::new(1),
        }
    }

    pub async fn handle_new_client(&self, stream: TcpStream) -> Result<()> {
        let addr = stream.peer_addr()?;
        let clients = Arc::clone(&self.clients);

        let (read_half, write_half) = stream.into_split();

        // Generate a new client ID
        let client_id = self.next_id.fetch_add(1, Ordering::SeqCst);

        // For now, assign a default user_id (1) - this should be replaced with actual authentication
        let connection = ChatRoomConnection {
            user_id: client_id as i32, // TODO: Replace with actual user authentication
            writer: write_half,
        };

        let mut clients_guard = clients.lock().await;
        clients_guard.insert(client_id, connection);
        drop(clients_guard);

        info!("New client connected: {} with ID: {}", addr, client_id);

        let mut connection_handler = crate::ConnectionHandler::new(Arc::clone(&clients));
        tokio::spawn(async move {
            if let Err(e) = connection_handler
                .handle_connection(client_id, read_half)
                .await
            {
                error!("Error handling connection from {}: {}", addr, e);
            }
        });

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chat_common::{async_message_stream::AsyncMessageStream, Message};
//     use std::time::Duration;
//     use tokio::net::TcpStream;
//     use tokio::sync::Mutex;
//     #[tokio::test]
//     async fn test_handle_new_client() {
//         let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
//         let addr = listener.local_addr().unwrap();
//         let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
//         let client_handler = ClientHandler::new(clients.clone());

//         tokio::spawn(async move {
//             let mut stream = TcpStream::connect(addr).await.unwrap();
//             stream
//                 .write_message(&Message::Text("Hello, server!".to_string()))
//                 .await
//                 .unwrap();
//         });

//         if let Ok((stream, _)) = listener.accept().await {
//             client_handler.handle_new_client(stream).await.unwrap();
//         }

//         tokio::time::sleep(Duration::from_millis(100)).await;
//         assert_eq!(clients.lock().await.len(), 1);
//     }
// }
