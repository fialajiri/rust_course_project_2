use anyhow::Result;

use chat_common::{
    async_message_stream::AsyncMessageStream,
    encryption::{file::EncryptedFileMetadata, message::EncryptedMessage, EncryptionService},
    error::ChatError,
    file_ops, Message,
};
use std::sync::Arc;
use tokio::io::BufReader;
use tracing::{error, info};

pub struct MessageHandler {
    encryption: Arc<EncryptionService>,
}

impl MessageHandler {
    pub fn new(encryption: Arc<EncryptionService>) -> Self {
        Self { encryption }
    }

    /// Handles incoming messages from the chat server.
    ///
    /// This function processes different types of messages:
    /// - Text messages: Decrypts and logs the content
    /// - System messages: Logs system notifications
    /// - File messages: Decrypts and saves received files
    /// - Image messages: Decrypts and saves received images
    /// - Error messages: Logs server errors
    /// - Auth messages: Handles authentication responses
    ///
    /// # Arguments
    /// * `stream` - A stream that implements AsyncMessageStream to read messages from
    ///
    /// # Returns
    /// * `Result<(), ChatError>` - Returns Ok(()) if successful, or a ChatError if message processing fails
    ///
    /// # Message Types
    ///
    /// ## Text Messages
    /// Text messages are encrypted and need to be decrypted using the encryption service.
    /// The decrypted content is logged using the info level.
    ///
    /// ## System Messages
    /// System messages are plain text notifications from the server.
    /// They are logged using the info level.
    ///
    /// ## File Messages
    /// File messages contain encrypted file data and metadata.
    /// The file is decrypted and saved to the local filesystem.
    ///
    /// ## Image Messages
    /// Image messages are similar to file messages but are specifically for image files.
    /// The image is decrypted and saved to the local filesystem.
    ///
    /// ## Error Messages
    /// Error messages contain an error code and description.
    /// They are logged using the error level.
    ///
    /// ## Auth Messages
    /// Auth messages are used for authentication responses.
    /// Success/failure status and messages are logged appropriately.
    ///
    /// # Examples
    /// ```
    /// use tokio::net::TcpStream;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let stream = TcpStream::connect("127.0.0.1:8080").await?;
    ///     let (read_half, _) = stream.into_split();
    ///     
    ///     let encryption = Arc::new(EncryptionService::new(&[0u8; 32]).unwrap());
    ///     let handler = MessageHandler::new(encryption);
    ///     handler.handle_incoming(read_half).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// The function can return the following types of errors:
    /// * `ChatError::IoError` - For I/O related errors (reading from stream, saving files)
    /// * `ChatError::EncryptionError` - For encryption/decryption failures
    /// * `ChatError::InvalidData` - For malformed messages or metadata
    ///
    /// # Notes
    /// * The function runs in a loop until the stream is closed or an error occurs
    /// * All errors are logged using the error level
    /// * Successful operations are logged using the info level
    /// * Files and images are saved to the local filesystem in the current directory
    pub async fn handle_incoming<S: AsyncMessageStream>(
        &self,
        mut stream: S,
    ) -> Result<(), ChatError> {
        while let Ok(message) = AsyncMessageStream::read_message(&mut stream).await {
            match message {
                Message::Text(encrypted) => {
                    // Decrypt the message
                    let encrypted: EncryptedMessage =
                        serde_json::from_str(&encrypted).map_err(|e| {
                            ChatError::SerializationError(format!(
                                "Failed to parse encrypted message: {}",
                                e
                            ))
                        })?;
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

                    let metadata: EncryptedFileMetadata = serde_json::from_value(metadata)
                        .map_err(|e| {
                            ChatError::SerializationError(format!(
                                "Failed to parse file metadata: {}",
                                e
                            ))
                        })?;

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

                    let metadata: EncryptedFileMetadata = serde_json::from_value(metadata)
                        .map_err(|e| {
                            ChatError::SerializationError(format!(
                                "Failed to parse image metadata: {}",
                                e
                            ))
                        })?;

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
    use super::*;
    use chat_common::{
        async_message_stream::AsyncMessageStream,
        encryption::EncryptionService,
        error::{ChatError, ErrorCode},
        file_ops, Message,
    };

    use async_trait::async_trait;
    use std::sync::Arc;

    struct TestStream {
        messages: Vec<Message>,
        current: usize,
    }

    impl TestStream {
        fn new(messages: Vec<Message>) -> Self {
            Self {
                messages,
                current: 0,
            }
        }
    }

    #[async_trait]
    impl AsyncMessageStream for TestStream {
        async fn read_message(&mut self) -> Result<Message, ChatError> {
            if self.current < self.messages.len() {
                let message = self.messages[self.current].clone();
                self.current += 1;
                Ok(message)
            } else {
                Err(ChatError::IoError(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "No more messages",
                )))
            }
        }

        async fn write_message(&mut self, _message: &Message) -> Result<(), ChatError> {
            Err(ChatError::IoError(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Cannot write messages in test stream",
            )))
        }
    }

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

    #[tokio::test]
    async fn test_message_handler_creation() {
        let encryption = Arc::new(EncryptionService::new(&[0u8; 32]).unwrap());
        let handler = MessageHandler::new(encryption.clone());
        assert!(Arc::ptr_eq(&handler.encryption, &encryption));
    }

    #[tokio::test]
    async fn test_handle_text_message() {
        let encryption = Arc::new(EncryptionService::new(&[0u8; 32]).unwrap());
        let handler = MessageHandler::new(encryption.clone());

        // Create a test encrypted message
        let test_text = "Hello, World!";
        let encrypted = encryption.message().encrypt(test_text).unwrap();
        let encrypted_json = serde_json::to_string(&encrypted).unwrap();

        // Create a test message
        let message = Message::Text(encrypted_json);
        let stream = TestStream::new(vec![message]);

        // Test handling the message
        let result = handler.handle_incoming(stream).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_system_message() {
        let encryption = Arc::new(EncryptionService::new(&[0u8; 32]).unwrap());
        let handler = MessageHandler::new(encryption);

        let message = Message::System("Test system message".to_string());
        let stream = TestStream::new(vec![message]);

        let result = handler.handle_incoming(stream).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_auth_response() {
        let encryption = Arc::new(EncryptionService::new(&[0u8; 32]).unwrap());
        let handler = MessageHandler::new(encryption);

        let message = Message::AuthResponse {
            success: true,
            token: Some("test_token".to_string()),
            message: "Authentication successful".to_string(),
        };
        let stream = TestStream::new(vec![message]);

        let result = handler.handle_incoming(stream).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_error_message() {
        let encryption = Arc::new(EncryptionService::new(&[0u8; 32]).unwrap());
        let handler = MessageHandler::new(encryption);

        let message = Message::Error {
            code: ErrorCode::PermissionDenied,
            message: "Access denied".to_string(),
        };
        let stream = TestStream::new(vec![message]);

        let result = handler.handle_incoming(stream).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_multiple_messages() {
        let encryption = Arc::new(EncryptionService::new(&[0u8; 32]).unwrap());
        let handler = MessageHandler::new(encryption.clone());

        // Create a sequence of different message types
        let messages = vec![
            Message::System("Server starting".to_string()),
            Message::Text(
                serde_json::to_string(&encryption.message().encrypt("Hello").unwrap()).unwrap(),
            ),
            Message::System("User joined".to_string()),
            Message::Error {
                code: ErrorCode::InvalidInput,
                message: "Invalid command".to_string(),
            },
        ];

        let stream = TestStream::new(messages);
        let result = handler.handle_incoming(stream).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_invalid_encrypted_message() {
        let encryption = Arc::new(EncryptionService::new(&[0u8; 32]).unwrap());
        let handler = MessageHandler::new(encryption);

        // Create a message with invalid encrypted data
        let message = Message::Text("invalid json".to_string());
        let stream = TestStream::new(vec![message]);

        let result = handler.handle_incoming(stream).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ChatError::SerializationError(_)
        ));
    }
}
