//! Message handling service for the chat server.
//!
//! This module handles incoming messages, processes them, and manages client connections.
//! It handles various types of messages including text, files, images, and system messages,
//! with appropriate encryption/decryption for secure communication.

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

use super::processor::MessageProcessor;

/// Service responsible for handling incoming messages and managing client connections.
///
/// The `MessageService` processes different types of messages, handles client disconnections,
/// and manages encrypted communication between clients.
#[derive(Clone)]
pub struct MessageService {
    clients: Clients,
    pool: Arc<DbPool>,
    encryption: Arc<EncryptionService>,
}

impl MessageService {
    /// Creates a new `MessageService` instance.
    ///
    /// # Arguments
    /// * `clients` - A shared collection of connected clients
    /// * `pool` - A shared database connection pool
    /// * `encryption` - A shared encryption service for secure communication
    pub fn new(clients: Clients, pool: Arc<DbPool>, encryption: Arc<EncryptionService>) -> Self {
        Self {
            clients,
            pool,
            encryption,
        }
    }

    /// Processes an incoming message using the message processor.
    ///
    /// # Arguments
    /// * `stream` - Optional TCP stream for reading additional data (used for file/image transfers)
    /// * `client_id` - The ID of the client sending the message
    /// * `message` - The message to process
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the message was processed successfully, Err otherwise
    pub async fn process_message(
        &self,
        stream: Option<&OwnedReadHalf>,
        client_id: usize,
        message: &Message,
    ) -> Result<()> {
        let processor = MessageProcessor::new(
            self.clients.clone(),
            Arc::clone(&self.pool),
            Arc::clone(&self.encryption),
        );
        processor.process(stream, client_id, message).await
    }

    /// Handles client disconnection and notifies other clients.
    ///
    /// # Arguments
    /// * `client_id` - The ID of the disconnecting client
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the disconnection was handled successfully, Err otherwise
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

    /// Processes binary data (files or images) with encryption/decryption.
    ///
    /// # Arguments
    /// * `name` - The name of the file/image
    /// * `metadata` - Encrypted metadata for the file/image
    /// * `data` - The encrypted binary data
    /// * `is_image` - Whether the data represents an image
    ///
    /// # Returns
    /// * `Result<Message>` - The processed message with re-encrypted data, or an error
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

    /// Handles an incoming message, processing it according to its type.
    ///
    /// # Arguments
    /// * `message` - The message to handle
    ///
    /// # Returns
    /// * `Result<Message>` - The processed message ready for broadcasting, or an error
    ///
    /// # Message Type Behavior
    /// * Text messages: Decrypted and re-encrypted for each recipient
    /// * File/Image messages: Decrypted, processed, and re-encrypted
    /// * System messages: Passed through without encryption
    /// * Auth messages: Passed through for processing
    /// * AuthResponse/Error messages: Logged as unexpected
    pub async fn handle_message(&self, message: Message) -> Result<Message> {
        match message {
            Message::Text(encrypted) => {
                // Decrypt incoming message
                let encrypted: EncryptedMessage = serde_json::from_str(&encrypted)?;
                let text = self.encryption.message().decrypt(&encrypted)?;

                // Re-encrypt for each recipient
                let encrypted = self.encryption.message().encrypt(&text)?;
                let encrypted_str = serde_json::to_string(&encrypted)?;

                Ok(Message::Text(encrypted_str))
            }
            Message::File {
                name,
                metadata,
                data,
            } => {
                let processed_message =
                    self.handle_binary_data(name, metadata, data, false).await?;
                Ok(processed_message)
            }
            Message::Image {
                name,
                metadata,
                data,
            } => {
                let processed_message = self.handle_binary_data(name, metadata, data, true).await?;
                Ok(processed_message)
            }
            Message::System(notification) => {
                // System messages are broadcast without encryption
                Ok(Message::System(notification))
            }
            Message::Auth { .. } => {
                // Auth messages are handled by the processor
                Ok(message)
            }
            Message::AuthResponse { .. } | Message::Error { .. } => {
                // These messages are typically sent by the server, not received
                warn!("Unexpected message type received from client");
                Ok(message)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chat_common::Message;
    use diesel_async::pooled_connection::deadpool::Pool;
    use diesel_async::pooled_connection::AsyncDieselConnectionManager;
    use diesel_async::AsyncPgConnection;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup_test_services() -> (Arc<DbPool>, Arc<EncryptionService>) {
        // Create a test encryption service with a test key
        let key = [0u8; 32]; // Test key (all zeros)
        let encryption = Arc::new(EncryptionService::new(&key).unwrap());

        // Create a minimal mock pool (we don't actually need it for these tests)
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
            "postgres://test:test@localhost/test",
        );
        let pool = Pool::builder(config).max_size(1).build().unwrap();
        let pool = Arc::new(pool);

        (pool, encryption)
    }

    #[tokio::test]
    async fn test_handle_text_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let (pool, encryption) = setup_test_services().await;
        let encryption_clone = Arc::clone(&encryption);

        let service = MessageService::new(clients, pool, encryption);

        // Create an encrypted message
        let encrypted = encryption_clone.message().encrypt("Test message").unwrap();
        let encrypted_str = serde_json::to_string(&encrypted).unwrap();
        let message = Message::Text(encrypted_str);

        let result = service.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_system_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let (pool, encryption) = setup_test_services().await;

        let service = MessageService::new(clients, pool, encryption);
        let message = Message::System("System notification".to_string());

        let result = service.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_auth_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let (pool, encryption) = setup_test_services().await;

        let service = MessageService::new(clients, pool, encryption);
        let message = Message::Auth {
            username: "test".to_string(),
            password: "test".to_string(),
        };

        let result = service.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_file_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let (pool, encryption) = setup_test_services().await;
        let encryption_clone = Arc::clone(&encryption);

        let service = MessageService::new(clients, pool, encryption);

        // Create test data and encrypt it
        let test_data = vec![1, 2, 3, 4, 5];
        let mut encrypted_data = Vec::new();
        let metadata = encryption_clone
            .file()
            .encrypt_stream(BufReader::new(&test_data[..]), &mut encrypted_data)
            .await
            .unwrap();

        let message = Message::File {
            name: "test.txt".to_string(),
            metadata: serde_json::to_value(metadata).unwrap(),
            data: encrypted_data,
        };

        let result = service.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_image_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let (pool, encryption) = setup_test_services().await;
        let encryption_clone = Arc::clone(&encryption);

        let service = MessageService::new(clients, pool, encryption);

        // Create test data and encrypt it
        let test_data = vec![1, 2, 3, 4, 5];
        let mut encrypted_data = Vec::new();
        let metadata = encryption_clone
            .file()
            .encrypt_stream(BufReader::new(&test_data[..]), &mut encrypted_data)
            .await
            .unwrap();

        let message = Message::Image {
            name: "test.png".to_string(),
            metadata: serde_json::to_value(metadata).unwrap(),
            data: encrypted_data,
        };

        let result = service.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_error_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let (pool, encryption) = setup_test_services().await;

        let service = MessageService::new(clients, pool, encryption);
        let message = Message::Error {
            code: chat_common::ErrorCode::PermissionDenied,
            message: "Test error".to_string(),
        };

        let result = service.handle_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_auth_response_message() {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let (pool, encryption) = setup_test_services().await;

        let service = MessageService::new(clients, pool, encryption);
        let message = Message::AuthResponse {
            success: true,
            token: Some("test_token".to_string()),
            message: "Authentication successful".to_string(),
        };

        let result = service.handle_message(message).await;
        assert!(result.is_ok());
    }
}
