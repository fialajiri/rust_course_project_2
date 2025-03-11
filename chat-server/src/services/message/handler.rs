use std::sync::Arc;

use crate::types::Clients;
use crate::utils::db_connection::DbPool;
use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::Message;
use tokio::net::tcp::OwnedReadHalf;
use tracing::info;

use super::processor::MessageProcessor;

#[derive(Clone)]
pub struct MessageService {
    clients: Clients,
    pool: Arc<DbPool>,
}

impl MessageService {
    pub fn new(clients: Clients, pool: Arc<DbPool>) -> Self {
        Self { clients, pool }
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
}
