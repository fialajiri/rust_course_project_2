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
