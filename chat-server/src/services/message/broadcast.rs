use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::Message;
use tracing::error;

use crate::types::Clients;

pub(super) struct MessageBroadcaster {
    clients: Clients,
}

impl MessageBroadcaster {
    pub fn new(clients: Clients) -> Self {
        Self { clients }
    }

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
