use crate::db_connection;
use crate::models::{MessageType, NewMessage};
use crate::types::Clients;
use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::Message;
use diesel::prelude::*;
use tokio::net::tcp::OwnedReadHalf;
use tracing::{error, info};

#[derive(Clone)]
pub struct MessageHandler {
    clients: Clients,
}

impl MessageHandler {
    pub fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub async fn process_message(
        &self,
        stream: Option<&OwnedReadHalf>,
        client_id: usize,
        message: &Message,
    ) -> Result<()> {
        // Get the user_id from the clients map
        let user_id = {
            let clients = self.clients.lock().await;
            clients
                .get(&client_id)
                .map(|conn| conn.user_id)
                .unwrap_or(1) // Default to user 1 if not found
        };

        // Save message to database
        let mut conn = db_connection::establish_connection();

        match message {
            Message::Text(content) => {
                let new_message = NewMessage {
                    sender_id: user_id,
                    message_type: MessageType::Text,
                    content: Some(content.clone()),
                    file_name: None,
                };
                diesel::insert_into(crate::schema::messages::table)
                    .values(&new_message)
                    .execute(&mut conn)?;
            }
            Message::File { name, .. } => {
                let new_message = NewMessage {
                    sender_id: user_id,
                    message_type: MessageType::File,
                    content: None,
                    file_name: Some(name.clone()),
                };
                diesel::insert_into(crate::schema::messages::table)
                    .values(&new_message)
                    .execute(&mut conn)?;
            }
            Message::Image { name, .. } => {
                let new_message = NewMessage {
                    sender_id: user_id,
                    message_type: MessageType::Image,
                    content: None,
                    file_name: Some(name.clone()),
                };
                diesel::insert_into(crate::schema::messages::table)
                    .values(&new_message)
                    .execute(&mut conn)?;
            }
            _ => {}
        }

        // Broadcast message to all clients
        self.broadcast_message(message).await?;

        // Send acknowledgment
        if let Some(read_stream) = stream {
            let ack_message = match message {
                Message::Text(_) => Some(Message::System("Message sent successfully".to_string())),
                Message::File { name, .. } => Some(Message::System(format!(
                    "File '{}' sent successfully",
                    name
                ))),
                Message::Image { name, .. } => Some(Message::System(format!(
                    "Image '{}' sent successfully",
                    name
                ))),
                _ => None,
            };

            if let Some(ack) = ack_message {
                let addr = read_stream.peer_addr()?;
                let mut clients = self.clients.lock().await;

                if let Some(client) = clients.get_mut(&client_id) {
                    if let Err(e) = client.writer.write_message(&ack).await {
                        error!("Failed to send acknowledgment: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn broadcast_message(&self, message: &Message) -> Result<()> {
        let mut clients = self.clients.lock().await;
        let mut failed_clients = Vec::new();

        for (client_id, connection) in clients.iter_mut() {
            if (connection.writer.write_message(message).await).is_err() {
                failed_clients.push(*client_id);
            }
        }

        for client_id in failed_clients {
            clients.remove(&client_id);
            error!("Removed disconnected client {}", client_id);
        }

        Ok(())
    }

    // Update handle_disconnect to work with client_id
    pub async fn handle_disconnect(&self, client_id: usize) -> Result<()> {
        let mut clients = self.clients.lock().await;
        clients.remove(&client_id);

        let disconnect_msg = Message::System("A client has disconnected".to_string());

        // Broadcast disconnect message to remaining clients
        for connection in clients.values_mut() {
            let _ = connection.writer.write_message(&disconnect_msg).await;
        }

        info!("Client {} disconnected", client_id);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chat_common::{async_message_stream::AsyncMessageStream, Message};
//     use std::collections::HashMap;
//     use std::sync::Arc;
//     use std::time::Duration;
//     use tokio::net::TcpStream;
//     use tokio::sync::Mutex;

//     #[tokio::test]
//     async fn test_process_text_message() {
//         let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
//         let handler = MessageHandler::new(clients);

//         let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
//         let addr = listener.local_addr().unwrap();

//         let client = TcpStream::connect(addr).await.unwrap();
//         let (mut read_half, mut write_half) = client.into_split();

//         if let Ok((stream, _)) = listener.accept().await {
//             let (mut server_read, mut server_write) = stream.into_split();
//             let message = Message::Text("Test message".to_string());

//             // Send message from client
//             write_half.write_message(&message).await.unwrap();

//             // Process message on server
//             if let Ok(received) = server_read.read_message().await {
//                 handler
//                     .process_message(Some(&server_read), 1, &received)
//                     .await
//                     .unwrap();
//             }

//             // Check for acknowledgment
//             if let Ok(ack) = read_half.read_message().await {
//                 match ack {
//                     Message::System(msg) => assert!(msg.contains("successfully")),
//                     _ => panic!("Expected system message"),
//                 }
//             }
//         }
//     }

//     #[tokio::test]
//     async fn test_handle_disconnect() {
//         let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
//         let handler = MessageHandler::new(clients.clone());

//         let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
//         let addr = listener.local_addr().unwrap();

//         let _client = TcpStream::connect(addr).await.unwrap();
//         let (server_stream, _) = listener.accept().await.unwrap();
//         let (_server_read, server_write) = server_stream.into_split();

//         let connection = ChatRoomConnection {
//             user_id: 1,
//             writer: server_write,
//         };
//         clients.lock().await.insert(1, connection);

//         handler.handle_disconnect(1).await.unwrap();
//         tokio::time::sleep(Duration::from_millis(100)).await;

//         assert_eq!(clients.lock().await.len(), 0);
//     }
// }
