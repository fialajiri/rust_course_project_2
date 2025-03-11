use crate::types::Clients;
use crate::utils::db_connection::DbPool;
use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use std::sync::Arc;
use tokio::net::tcp::OwnedReadHalf;
use tracing::error;

use super::message::handler::MessageService;

pub struct ConnectionService {
    clients: Clients,
    pool: Arc<DbPool>,
}

impl ConnectionService {
    pub fn new(clients: Clients, pool: Arc<DbPool>) -> Self {
        Self { clients, pool }
    }

    pub async fn handle_connection(
        &mut self,
        client_id: usize,
        mut stream: OwnedReadHalf,
    ) -> Result<()> {
        let addr = stream.peer_addr()?;
        let message_service = MessageService::new(self.clients.clone(), Arc::clone(&self.pool));

        while let Ok(message) = stream.read_message().await {
            if let Err(e) = message_service
                .process_message(Some(&stream), client_id, &message)
                .await
            {
                error!("Error processing message from {}: {}", addr, e);
                break;
            }
        }

        message_service.handle_disconnect(client_id).await?;
        Ok(())
    }
}
