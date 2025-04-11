//! Message processing service for the chat server.
//!
//! This module handles the processing of messages, including authentication,
//! message persistence, and message broadcasting to appropriate clients.

use std::sync::Arc;

use crate::models::message::{MessageType, NewMessage};
use crate::services::auth::AuthService;
use crate::types::{AuthState, Clients};
use crate::utils::db_connection::DbPool;
use crate::utils::metrics::Metrics;
use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::encryption::EncryptionService;
use chat_common::{ErrorCode, Message};
use diesel_async::RunQueryDsl;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::Mutex;
use tracing::{error, info};

use super::broadcast::MessageBroadcaster;

/// Service responsible for processing incoming messages and managing message flow.
///
/// The `MessageProcessor` handles message authentication, persistence, and broadcasting.
/// It ensures messages are properly saved to the database and delivered to appropriate clients.
pub(super) struct MessageProcessor {
    clients: Clients,
    pool: Arc<DbPool>,
    encryption: Arc<EncryptionService>,
    metrics: Arc<Mutex<Metrics>>,
}

impl MessageProcessor {
    /// Creates a new `MessageProcessor` instance.
    ///
    /// # Arguments
    /// * `clients` - A shared collection of connected clients
    /// * `pool` - A shared database connection pool
    /// * `encryption` - A shared encryption service for secure communication
    /// * `metrics` - A shared metrics service for tracking message processing
    pub fn new(
        clients: Clients,
        pool: Arc<DbPool>,
        encryption: Arc<EncryptionService>,
        metrics: Arc<Mutex<Metrics>>,
    ) -> Self {
        Self {
            clients,
            pool,
            encryption,
            metrics,
        }
    }

    /// Processes an incoming message, handling authentication and broadcasting.
    ///
    /// # Arguments
    /// * `stream` - Optional TCP stream for reading additional data (used for file/image transfers)
    /// * `client_id` - The ID of the client sending the message
    /// * `message` - The message to process
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the message was processed successfully, Err otherwise
    ///
    /// # Message Processing Flow
    /// 1. Authentication messages are handled separately
    /// 2. For other messages, client authentication is verified
    /// 3. If authenticated:
    ///    - Message is saved to database
    ///    - Acknowledgment is sent to sender
    ///    - Message is broadcast to other authenticated clients
    /// 4. If not authenticated:
    ///    - Error message is sent to client
    pub async fn process(
        &self,
        _stream: Option<&OwnedReadHalf>,
        client_id: usize,
        message: &Message,
    ) -> Result<()> {
        if let Message::Auth { username, password } = message {
            return self.handle_auth(client_id, username, password).await;
        }

        let (is_authenticated, user_id) = self.get_auth_status(client_id).await?;

        if !is_authenticated {
            return self.handle_unauthenticated(client_id).await;
        }

        // Save message to database
        self.save_message_to_db(message, user_id).await?;

        // Increment message counter
        self.metrics.lock().await.messages_sent.inc();

        // First send acknowledgment to the sender
        self.send_acknowledgment(client_id, message).await?;

        // Then broadcast to all other authenticated users
        let broadcaster = MessageBroadcaster::new(self.clients.clone());
        broadcaster
            .broadcast_message(message, Some(client_id))
            .await?;

        Ok(())
    }

    /// Retrieves the authentication status and user ID for a client.
    ///
    /// # Arguments
    /// * `client_id` - The ID of the client to check
    ///
    /// # Returns
    /// * `Result<(bool, i32)>` - Tuple containing (is_authenticated, user_id)
    async fn get_auth_status(&self, client_id: usize) -> Result<(bool, i32)> {
        let clients = self.clients.lock().await;
        let client = clients
            .get(&client_id)
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        Ok((
            client.is_authenticated(),
            client.user_id.unwrap_or_default(),
        ))
    }

    /// Handles unauthenticated client messages by sending an error response.
    ///
    /// # Arguments
    /// * `client_id` - The ID of the unauthenticated client
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the error was sent successfully, Err otherwise
    async fn handle_unauthenticated(&self, client_id: usize) -> Result<()> {
        let mut clients = self.clients.lock().await;
        if let Some(client) = clients.get_mut(&client_id) {
            let error = Message::Error {
                code: ErrorCode::PermissionDenied,
                message: "Authentication required".to_string(),
            };
            client.writer.write_message(&error).await?;
        }
        Ok(())
    }

    /// Saves a message to the database.
    ///
    /// # Arguments
    /// * `message` - The message to save
    /// * `user_id` - The ID of the user sending the message
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the message was saved successfully, Err otherwise
    async fn save_message_to_db(&self, message: &Message, user_id: i32) -> Result<()> {
        let conn = &mut *self.pool.get().await?;

        let new_message = match message {
            Message::Text(content) => {
                // Decrypt the text message before saving
                let encrypted: chat_common::encryption::message::EncryptedMessage =
                    serde_json::from_str(content)?;
                let decrypted = self.encryption.message().decrypt(&encrypted)?;

                Some(NewMessage {
                    sender_id: user_id,
                    message_type: MessageType::Text,
                    content: Some(decrypted),
                    file_name: None,
                })
            }
            Message::File { name, .. } => Some(NewMessage {
                sender_id: user_id,
                message_type: MessageType::File,
                content: None,
                file_name: Some(name.clone()),
            }),
            Message::Image { name, .. } => Some(NewMessage {
                sender_id: user_id,
                message_type: MessageType::Image,
                content: None,
                file_name: Some(name.clone()),
            }),
            _ => None,
        };

        if let Some(msg) = new_message {
            diesel::insert_into(crate::schema::messages::table)
                .values(&msg)
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    /// Sends an acknowledgment message to the sender.
    ///
    /// # Arguments
    /// * `client_id` - The ID of the client to send the acknowledgment to
    /// * `message` - The original message that was processed
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the acknowledgment was sent successfully, Err otherwise
    async fn send_acknowledgment(&self, client_id: usize, message: &Message) -> Result<()> {
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
            let mut clients = self.clients.lock().await;
            if let Some(client) = clients.get_mut(&client_id) {
                if let Err(e) = client.writer.write_message(&ack).await {
                    error!("Failed to send acknowledgment: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handles client authentication.
    ///
    /// # Arguments
    /// * `client_id` - The ID of the client to authenticate
    /// * `username` - The username provided for authentication
    /// * `password` - The password provided for authentication
    ///
    /// # Returns
    /// * `Result<()>` - Ok if authentication was processed successfully, Err otherwise
    async fn handle_auth(&self, client_id: usize, username: &str, password: &str) -> Result<()> {
        let auth_service = AuthService::new(self.pool.clone());

        match auth_service.authenticate(username, password).await? {
            Some((user_id, token)) => {
                let mut clients = self.clients.lock().await;
                if let Some(client) = clients.get_mut(&client_id) {
                    client.user_id = Some(user_id);
                    client.auth_state = AuthState::Authenticated {
                        user_id,
                        token: token.clone(),
                    };

                    let response = Message::AuthResponse {
                        success: true,
                        token: Some(token),
                        message: "Authentication successful".to_string(),
                    };

                    info!("Client {} authenticated successfully", client_id);

                    client.writer.write_message(&response).await?;
                }
            }
            None => {
                let mut clients = self.clients.lock().await;
                if let Some(client) = clients.get_mut(&client_id) {
                    let response = Message::AuthResponse {
                        success: false,
                        token: None,
                        message: "Invalid credentials".to_string(),
                    };

                    info!("Client {} authentication failed", client_id);

                    client.writer.write_message(&response).await?;
                }
            }
        }
        Ok(())
    }
}
