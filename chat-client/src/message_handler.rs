use anyhow::Result;
use chat_common::{
    async_message_stream::AsyncMessageStream,
    encryption::{file::EncryptedFileMetadata, message::EncryptedMessage, EncryptionService},
    file_ops, Message,
};
use std::sync::Arc;
use tokio::{io::BufReader, net::tcp::OwnedReadHalf};
use tracing::{error, info};

pub struct MessageHandler {
    encryption: Arc<EncryptionService>,
}

impl MessageHandler {
    pub fn new(encryption: Arc<EncryptionService>) -> Self {
        Self { encryption }
    }

    pub async fn handle_incoming(&self, mut stream: OwnedReadHalf) -> Result<()> {
        while let Ok(message) = AsyncMessageStream::read_message(&mut stream).await {
            match &message {
                Message::File { data, .. } | Message::Image { data, .. } => {
                    info!(
                        "Received message (first 100 bytes): {:?}",
                        &data[..100.min(data.len())]
                    );
                }
                _ => info!("Received message: {:?}", message),
            }
            match message {
                Message::Text(encrypted) => {
                    // Decrypt the message
                    let encrypted: EncryptedMessage = serde_json::from_str(&encrypted)?;
                    match self.encryption.message().decrypt(&encrypted) {
                        Ok(text) => info!("Received: {}", text),
                        Err(e) => error!("Failed to decrypt message: {}", e),
                    }
                }
                Message::System(notification) => {
                    info!("System: {}", notification);
                }
                Message::File {
                    name,
                    metadata,
                    data,
                } => {
                    info!("Receiving encrypted file: {}", name);
                    let mut buffer = Vec::new();

                    let metadata: EncryptedFileMetadata = serde_json::from_value(metadata)?;

                    self.encryption
                        .file()
                        .decrypt_stream(BufReader::new(&data[..]), &mut buffer, &metadata)
                        .await?;

                    if let Err(e) = file_ops::save_file(&name, buffer).await {
                        error!("{}", e);
                    }
                }
                Message::Image {
                    name,
                    metadata,
                    data,
                } => {
                    info!("Receiving image: {}", name);
                    let mut buffer = Vec::new();

                    let metadata: EncryptedFileMetadata = serde_json::from_value(metadata)?;

                    self.encryption
                        .file()
                        .decrypt_stream(BufReader::new(&data[..]), &mut buffer, &metadata)
                        .await?;

                    info!("Decrypted image size: {}", buffer.len());
                    if let Err(e) = file_ops::save_image(&name, buffer).await {
                        error!("Failed to save image: {}", e);
                    }
                }
                Message::Error { code, message } => {
                    error!("Server error [{}]: {}", format!("{:?}", code), message);
                }
                Message::AuthResponse {
                    success,
                    token: _token,
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
