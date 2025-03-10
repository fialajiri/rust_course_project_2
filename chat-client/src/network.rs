use tokio::net::tcp::OwnedReadHalf;
use tracing::error;

use crate::message_handler::handle_incoming;

pub fn spawn_receiver_task(stream: OwnedReadHalf) {
    tokio::spawn(async move {
        if let Err(e) = handle_incoming(stream).await {
            error!("Error handling incoming messages: {}", e);
        }
    });
}
