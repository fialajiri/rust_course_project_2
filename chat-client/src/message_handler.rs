use anyhow::Result;
use chat_common::{file_ops, Message, MessageStream};
use std::net::TcpStream;
use tracing::{error, info};

pub fn handle_incoming(stream: &mut TcpStream) -> Result<()> {
    while let Ok(message) = stream.read_message() {
        println!("Received message: {:?}", message);
        match message {
            Message::Text(text) => {
                info!("Received: {}", text);
            }
            Message::System(notification) => {
                info!("System: {}", notification);
            }
            Message::File { name, data } => {
                info!("Receiving file: {}", name);
                if let Err(e) = file_ops::save_file(&name, data) {
                    error!("Failed to save file {}: {}", name, e);
                }
            }
            Message::Image { name, data } => {
                info!("Receiving image: {}", name);
                if let Err(e) = file_ops::save_image(&name, data) {
                    error!("Failed to save image {}: {}", name, e);
                }
            }
            Message::Error { code, message } => {
                error!("Server error [{}]: {}", format!("{:?}", code), message);
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

    #[test]
    fn test_create_error_message() {
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
