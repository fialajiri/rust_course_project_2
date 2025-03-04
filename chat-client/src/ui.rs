use anyhow::Result;
use chat_common::MessageStream;
use std::io::{self, BufRead};
use std::net::TcpStream;

use crate::commands::{parse_command, process_command, Command};

pub fn run_input_loop(mut stream: TcpStream) -> Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        let command = parse_command(&line);

        // Handle quit command directly
        if matches!(command, Command::Quit) {
            break;
        }

        // Process other commands
        if let Ok(Some(message)) = process_command(command) {
            stream.write_message(&message)?;
        }
    }

    Ok(())
}
