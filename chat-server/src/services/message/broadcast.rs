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

    pub async fn broadcast_message(&self, message: &Message) -> Result<()> {
        match message {
            Message::Text(_)
            | Message::File { .. }
            | Message::Image { .. }
            | Message::System(_) => {
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
            }
            // Don't broadcast auth-related messages
            Message::Auth { .. } | Message::AuthResponse { .. } | Message::Error { .. } => {}
        }

        Ok(())
    }
}
