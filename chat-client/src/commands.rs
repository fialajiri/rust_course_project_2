use anyhow::Result;
use chat_common::encryption::EncryptionService;
use chat_common::file_ops;
use chat_common::Message;
use std::sync::Arc;
use tracing::{error, warn};

pub enum Command {
    Text(String),
    File(String),
    Image(String),
    Auth { username: String, password: String },
    Quit,
    Invalid,
}

pub struct CommandProcessor {
    encryption: Arc<EncryptionService>,
}

impl CommandProcessor {
    pub fn new(encryption: Arc<EncryptionService>) -> Self {
        Self { encryption }
    }

    /// Parses a command string into a Command enum.
    ///
    /// The function supports the following commands:
    /// - `.quit` - Exits the chat
    /// - `.login <username> <password>` - Authenticates the user
    /// - `.file <path>` - Sends a file
    /// - `.image <path>` - Sends an image
    /// - Any other text (without leading dot) is treated as a text message
    ///
    /// # Arguments
    /// * `input` - The command string to parse
    ///
    /// # Returns
    /// A Command enum variant representing the parsed command
    pub fn parse_command(&self, input: &str) -> Command {
        if input == ".quit" {
            return Command::Quit;
        }

        if input.starts_with(".login ") {
            let args = input.trim_start_matches(".login ").trim();
            let parts: Vec<&str> = args.split_whitespace().collect();
            if parts.len() == 2 {
                return Command::Auth {
                    username: parts[0].to_string(),
                    password: parts[1].to_string(),
                };
            }
            return Command::Invalid;
        }

        if input.starts_with(".file ") {
            let path = input.trim_start_matches(".file ").trim();
            if path.is_empty() {
                return Command::Invalid;
            }
            return Command::File(path.to_string());
        }

        if input.starts_with(".image ") {
            let path = input.trim_start_matches(".image ").trim();
            if path.is_empty() {
                return Command::Invalid;
            }
            return Command::Image(path.to_string());
        }

        if input.starts_with('.') {
            return Command::Invalid;
        }

        Command::Text(input.to_string())
    }

    pub async fn process_command(&self, command: Command) -> Result<Option<Message>> {
        match command {
            Command::Text(text) => {
                // Encrypt the text message
                let encrypted = self.encryption.message().encrypt(&text)?;
                Ok(Some(Message::Text(serde_json::to_string(&encrypted)?)))
            }
            Command::File(path) => self.process_file_command(".file", &path).await,
            Command::Image(path) => self.process_file_command(".image", &path).await,
            Command::Auth { username, password } => Ok(Some(Message::Auth { username, password })),
            Command::Quit => Ok(None),
            Command::Invalid => {
                warn!("Invalid command format");
                Ok(None)
            }
        }
    }

    async fn process_file_command(&self, command: &str, path: &str) -> Result<Option<Message>> {
        match file_ops::process_file_command(command, path, Some(self.encryption.clone())).await {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => {
                error!("{}", e);
                Ok(Some(file_ops::create_error_message(&e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chat_common::encryption::EncryptionService;

    fn create_processor() -> CommandProcessor {
        let test_key = [0u8; 32]; // Test key for encryption
        CommandProcessor::new(Arc::new(EncryptionService::new(&test_key).unwrap()))
    }

    #[test]
    fn test_parse_quit_command() {
        let processor = create_processor();
        assert!(matches!(processor.parse_command(".quit"), Command::Quit));
    }

    #[test]
    fn test_parse_login_command() {
        let processor = create_processor();
        let cmd = processor.parse_command(".login user pass");
        match cmd {
            Command::Auth { username, password } => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
            }
            _ => panic!("Expected Auth command"),
        }
    }

    #[test]
    fn test_parse_invalid_login_command() {
        let processor = create_processor();
        assert!(matches!(
            processor.parse_command(".login"),
            Command::Invalid
        ));
        assert!(matches!(
            processor.parse_command(".login user"),
            Command::Invalid
        ));
        assert!(matches!(
            processor.parse_command(".login user pass extra"),
            Command::Invalid
        ));
    }

    #[test]
    fn test_parse_file_command() {
        let processor = create_processor();
        let cmd = processor.parse_command(".file test.txt");
        match cmd {
            Command::File(path) => assert_eq!(path, "test.txt"),
            _ => panic!("Expected File command"),
        }
    }

    #[test]
    fn test_parse_empty_file_command() {
        let processor = create_processor();
        assert!(matches!(processor.parse_command(".file"), Command::Invalid));
        assert!(matches!(
            processor.parse_command(".file "),
            Command::Invalid
        ));
    }

    #[test]
    fn test_parse_image_command() {
        let processor = create_processor();
        let cmd = processor.parse_command(".image photo.jpg");
        match cmd {
            Command::Image(path) => assert_eq!(path, "photo.jpg"),
            _ => panic!("Expected Image command"),
        }
    }

    #[test]
    fn test_parse_empty_image_command() {
        let processor = create_processor();
        assert!(matches!(
            processor.parse_command(".image"),
            Command::Invalid
        ));
        assert!(matches!(
            processor.parse_command(".image "),
            Command::Invalid
        ));
    }

    #[test]
    fn test_parse_text_command() {
        let processor = create_processor();
        let cmd = processor.parse_command("Hello, World!");
        match cmd {
            Command::Text(text) => assert_eq!(text, "Hello, World!"),
            _ => panic!("Expected Text command"),
        }
    }

    #[test]
    fn test_parse_invalid_command() {
        let processor = create_processor();
        assert!(matches!(
            processor.parse_command(".invalid"),
            Command::Invalid
        ));
        assert!(matches!(
            processor.parse_command(".unknown"),
            Command::Invalid
        ));
        assert!(matches!(processor.parse_command(".file"), Command::Invalid));
        assert!(matches!(
            processor.parse_command(".image"),
            Command::Invalid
        ));
    }
}
