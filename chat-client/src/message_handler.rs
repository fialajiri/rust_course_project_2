use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::{file_ops, Message};
use tokio::net::tcp::OwnedReadHalf;
use tracing::{error, info};

pub async fn handle_incoming(mut stream: OwnedReadHalf) -> Result<()> {
    while let Ok(message) = AsyncMessageStream::read_message(&mut stream).await {
        match message {
            Message::Text(text) => {
                info!("Received: {}", text);
            }
            Message::System(notification) => {
                info!("System: {}", notification);
            }
            Message::File { name, data } => {
                info!("Receiving file: {}", name);
                if let Err(e) = file_ops::save_file(&name, data).await {
                    error!("{}", e);
                }
            }
            Message::Image { name, data } => {
                info!("Receiving image: {}", name);
                if let Err(e) = file_ops::save_image(&name, data).await {
                    error!("{}", e);
                }
            }
            Message::Error { code, message } => {
                error!("Server error [{}]: {}", format!("{:?}", code), message);
            }
            Message::AuthResponse {
                success,
                token,
                message,
            } => {
                if success {
                    info!("Authentication successful: {}", message);
                } else {
                    error!("Authentication failed: {}", message);
                }
            }
            Message::Auth { .. } => {
                // Client doesn't need to handle incoming Auth messages
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chat_common::{
        error::{ChatError, ErrorCode},
        file_ops, Message,
    };

    #[tokio::test]
    async fn test_create_error_message() {
        let error = ChatError::NotFound("test.txt".to_string());
        let message = file_ops::create_error_message(&error);

        match message {
            Message::Error { code, message } => {
                assert_eq!(code, ErrorCode::FileNotFound);
                assert_eq!(message, "File not found: test.txt");
            }
            _ => panic!("Expected Error message"),
        }
    }
}
