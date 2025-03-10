use anyhow::Result;
use chat_common::async_message_stream::AsyncMessageStream;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::tcp::OwnedWriteHalf,
};

use crate::commands::{parse_command, process_command, Command};

pub async fn run_input_loop(mut stream: OwnedWriteHalf) -> Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }

        let command = parse_command(line.trim());

        // Handle quit command directly
        if matches!(command, Command::Quit) {
            break;
        }

        // Process other commands
        if let Ok(Some(message)) = process_command(command).await {
            AsyncMessageStream::write_message(&mut stream, &message).await?;
        }
    }

    Ok(())
}
