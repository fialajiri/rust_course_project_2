use std::sync::Arc;

use crate::models::message::{MessageType, NewMessage};
use crate::services::auth::AuthService;
use crate::types::{AuthState, Clients};
use crate::utils::db_connection::DbPool;
use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::{ErrorCode, Message};
use diesel_async::RunQueryDsl;
use tokio::net::tcp::OwnedReadHalf;
use tracing::{error, info};

use super::broadcast::MessageBroadcaster;

pub(super) struct MessageProcessor {
    clients: Clients,
    pool: Arc<DbPool>,
}

impl MessageProcessor {
    pub fn new(clients: Clients, pool: Arc<DbPool>) -> Self {
        Self { clients, pool }
    }

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

        // First send acknowledgment to the sender
        self.send_acknowledgment(client_id, message).await?;

        // Then broadcast to all other authenticated users
        let broadcaster = MessageBroadcaster::new(self.clients.clone());
        broadcaster
            .broadcast_message(message, Some(client_id))
            .await?;

        Ok(())
    }

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

    async fn save_message_to_db(&self, message: &Message, user_id: i32) -> Result<()> {
        let conn = &mut *self.pool.get().await?;

        let new_message = match message {
            Message::Text(content) => Some(NewMessage {
                sender_id: user_id,
                message_type: MessageType::Text,
                content: Some(content.clone()),
                file_name: None,
            }),
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
