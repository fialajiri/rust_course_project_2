use std::sync::Arc;

use crate::types::Clients;
use crate::utils::db_connection::DbPool;
use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::encryption::file::EncryptedFileMetadata;
use chat_common::encryption::message::EncryptedMessage;
use chat_common::encryption::EncryptionService;
use chat_common::Message;
use tokio::io::BufReader;
use tokio::net::tcp::OwnedReadHalf;
use tracing::{info, warn};

use super::broadcast::MessageBroadcaster;
use super::processor::MessageProcessor;

#[derive(Clone)]
pub struct MessageService {
    clients: Clients,
    pool: Arc<DbPool>,
    encryption: Arc<EncryptionService>,
}

impl MessageService {
    pub fn new(clients: Clients, pool: Arc<DbPool>, encryption: Arc<EncryptionService>) -> Self {
        Self {
            clients,
            pool,
            encryption,
        }
    }

    pub async fn process_message(
        &self,
        stream: Option<&OwnedReadHalf>,
        client_id: usize,
        message: &Message,
    ) -> Result<()> {
        let processor = MessageProcessor::new(self.clients.clone(), Arc::clone(&self.pool));
        processor.process(stream, client_id, message).await
    }

    pub async fn handle_disconnect(&self, client_id: usize) -> Result<()> {
        let mut clients = self.clients.lock().await;
        clients.remove(&client_id);

        // TODO: get the username of the disconnected client
        let disconnect_msg = Message::System("A client has disconnected".to_string());

        // Broadcast disconnect message to remaining clients
        for connection in clients.values_mut() {
            let _ = connection.writer.write_message(&disconnect_msg).await;
        }

        info!("Client {} disconnected", client_id);
        Ok(())
    }

    // Helper method to handle both File and Image messages
    async fn handle_binary_data(
        &self,
        name: String,
        metadata: serde_json::Value,
        data: Vec<u8>,
        is_image: bool,
    ) -> Result<Message> {
        // Decrypt the incoming data
        let mut decrypted = Vec::new();
        let metadata_typed: EncryptedFileMetadata = serde_json::from_value(metadata)?;

        self.encryption
            .file()
            .decrypt_stream(BufReader::new(&data[..]), &mut decrypted, &metadata_typed)
            .await?;

        // Re-encrypt for broadcast
        let mut encrypted_data = Vec::new();
        let new_metadata = self
            .encryption
            .file()
            .encrypt_stream(BufReader::new(&decrypted[..]), &mut encrypted_data)
            .await?;

        // Create the appropriate message type
        if is_image {
            Ok(Message::Image {
                name,
                metadata: serde_json::to_value(new_metadata)?,
                data: encrypted_data,
            })
        } else {
            Ok(Message::File {
                name,
                metadata: serde_json::to_value(new_metadata)?,
                data: encrypted_data,
            })
        }
    }

    pub async fn handle_message(&self, client_id: usize, message: Message) -> Result<()> {
        let broadcaster = MessageBroadcaster::new(self.clients.clone());

        match message {
            Message::Text(encrypted) => {
                // Decrypt incoming message
                let encrypted: EncryptedMessage = serde_json::from_str(&encrypted)?;
                let text = self.encryption.message().decrypt(&encrypted)?;

                // Re-encrypt for each recipient
                let encrypted = self.encryption.message().encrypt(&text)?;
                let encrypted_str = serde_json::to_string(&encrypted)?;

                // Broadcast re-encrypted message
                broadcaster
                    .broadcast_message(&Message::Text(encrypted_str))
                    .await?;
            }
            Message::File {
                name,
                metadata,
                data,
            } => {
                let processed_message =
                    self.handle_binary_data(name, metadata, data, false).await?;
                broadcaster.broadcast_message(&processed_message).await?;
            }
            Message::Image {
                name,
                metadata,
                data,
            } => {
                let processed_message = self.handle_binary_data(name, metadata, data, true).await?;
                broadcaster.broadcast_message(&processed_message).await?;
            }
            Message::System(notification) => {
                // System messages are broadcast without encryption
                broadcaster
                    .broadcast_message(&Message::System(notification))
                    .await?;
            }
            Message::Auth { .. } => {
                // Auth messages are handled by the processor
                self.process_message(None, client_id, &message).await?;
            }
            Message::AuthResponse { .. } | Message::Error { .. } => {
                // These messages are typically sent by the server, not received
                warn!("Unexpected message type received from client");
            }
        }
        Ok(())
    }
}
