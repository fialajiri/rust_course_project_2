use crate::types::Clients;
use crate::MessageHandler;
use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use tokio::net::tcp::OwnedReadHalf;
use tracing::error;

pub struct ConnectionHandler {
    clients: Clients,
}

impl ConnectionHandler {
    pub fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub async fn handle_connection(
        &mut self,
        client_id: usize,
        mut stream: OwnedReadHalf,
    ) -> Result<()> {
        let addr = stream.peer_addr()?;
        let message_handler = MessageHandler::new(self.clients.clone());

        while let Ok(message) = stream.read_message().await {
            if let Err(e) = message_handler
                .process_message(Some(&stream), client_id, &message)
                .await
            {
                error!("Error processing message from {}: {}", addr, e);
                break;
            }
        }

        message_handler.handle_disconnect(client_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chat_common::{async_message_stream::AsyncMessageStream, Message};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::net::TcpStream;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_handle_connection() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

        let mut handler = ConnectionHandler::new(clients.clone());

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let (read_half, _) = stream.into_split();
                handler.handle_connection(0, read_half).await.unwrap();
            }
        });

        let mut client = TcpStream::connect(addr).await.unwrap();
        let test_message = Message::Text("Hello".to_string());
        client.write_message(&test_message).await.unwrap();

        // Read acknowledgment
        if let Ok(response) = client.read_message().await {
            match response {
                Message::System(msg) => assert!(msg.contains("successfully")),
                _ => panic!("Expected system message"),
            }
        }
    }
}
