use chat_common::encryption::EncryptionService;
use std::sync::Arc;
use tokio::net::tcp::OwnedReadHalf;
use tracing::error;

use crate::message_handler::MessageHandler;

pub fn spawn_receiver_task(stream: OwnedReadHalf, encryption: Arc<EncryptionService>) {
    tokio::spawn(async move {
        let handler = MessageHandler::new(encryption);
        if let Err(e) = handler.handle_incoming(stream).await {
            error!("Error handling incoming messages: {}", e);
        }
    });
}
