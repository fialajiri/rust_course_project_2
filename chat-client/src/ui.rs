use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use chat_common::encryption::EncryptionService;
use std::sync::Arc;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::tcp::OwnedWriteHalf,
};

use crate::commands::{Command, CommandProcessor};

pub async fn run_input_loop(
    mut stream: OwnedWriteHalf,
    encryption: Arc<EncryptionService>,
) -> Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    let processor = CommandProcessor::new(encryption);

    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }

        let command = processor.parse_command(line.trim());

        // Handle quit command directly
        if matches!(command, Command::Quit) {
            break;
        }

        // Process other commands
        if let Ok(Some(message)) = processor.process_command(command).await {
            AsyncMessageStream::write_message(&mut stream, &message).await?;
        }
    }

    Ok(())
}
