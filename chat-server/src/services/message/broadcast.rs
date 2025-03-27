//! Message broadcasting service for the chat server.
//!
//! This module handles broadcasting messages to connected clients based on various criteria
//! such as authentication status and sender information.

use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::Message;
use tracing::error;

use crate::types::Clients;

/// A service responsible for broadcasting messages to connected clients.
///
/// The `MessageBroadcaster` handles different types of messages and ensures they are
/// delivered to the appropriate clients based on message type and client authentication status.
pub(super) struct MessageBroadcaster {
    clients: Clients,
}

impl MessageBroadcaster {
    /// Creates a new `MessageBroadcaster` instance.
    ///
    /// # Arguments
    /// * `clients` - A shared collection of connected clients
    pub fn new(clients: Clients) -> Self {
        Self { clients }
    }

    /// Sends a message to clients that match the given predicate.
    ///
    /// # Arguments
    /// * `message` - The message to send
    /// * `should_send` - A predicate function that determines if a message should be sent to a client
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the operation completed successfully, Err otherwise
    ///
    /// # Note
    /// This method automatically removes disconnected clients from the client list.
    async fn send_to_clients<F>(&self, message: &Message, should_send: F) -> Result<()>
    where
        F: Fn(&mut crate::types::ChatRoomConnection) -> bool,
    {
        let mut clients = self.clients.lock().await;
        let mut failed_clients = Vec::new();

        for (client_id, connection) in clients.iter_mut() {
            if should_send(connection) && (connection.writer.write_message(message).await).is_err()
            {
                failed_clients.push(*client_id);
            }
        }

        for client_id in failed_clients {
            clients.remove(&client_id);
            error!("Removed disconnected client {}", client_id);
        }

        Ok(())
    }

    /// Broadcasts a message to appropriate clients based on message type and sender.
    ///
    /// # Arguments
    /// * `message` - The message to broadcast
    /// * `sender_id` - The ID of the message sender (if any)
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the operation completed successfully, Err otherwise
    ///
    /// # Message Type Behavior
    /// * Text/File/Image messages: Only sent to authenticated clients, excluding the sender
    /// * System messages: Sent to all clients, excluding the sender
    /// * Auth/AuthResponse/Error messages: Not broadcast (handled separately)
    pub async fn broadcast_message(
        &self,
        message: &Message,
        sender_id: Option<usize>,
    ) -> Result<()> {
        match message {
            Message::Text(_) | Message::File { .. } | Message::Image { .. } => {
                // Only send to authenticated clients, excluding the sender
                self.send_to_clients(message, |connection| {
                    connection.is_authenticated()
                        && Some(connection.user_id.unwrap_or_default() as usize) != sender_id
                })
                .await
            }
            Message::System(_) => {
                // Send to all clients, excluding the sender
                self.send_to_clients(message, |connection| {
                    Some(connection.user_id.unwrap_or_default() as usize) != sender_id
                })
                .await
            }
            // Don't broadcast auth-related messages
            Message::Auth { .. } | Message::AuthResponse { .. } | Message::Error { .. } => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chat_common::Message;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_broadcast_text_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let broadcaster = MessageBroadcaster::new(clients.clone());

        let message = Message::Text("Hello, World!".to_string());
        let result = broadcaster.broadcast_message(&message, Some(1)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_system_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let broadcaster = MessageBroadcaster::new(clients.clone());

        let message = Message::System("System message".to_string());
        let result = broadcaster.broadcast_message(&message, Some(1)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_auth_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let broadcaster = MessageBroadcaster::new(clients.clone());

        let message = Message::Auth {
            username: "test".to_string(),
            password: "test".to_string(),
        };
        let result = broadcaster.broadcast_message(&message, Some(1)).await;

        assert!(result.is_ok());
    }
}
