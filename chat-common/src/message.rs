use crate::error::ErrorCode;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::net::TcpStream;

/// Represents different types of messages that can be sent between client and server
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Message {
    /// Simple text message
    Text(String),

    /// System notification message
    System(String),

    /// File transfer message
    File { name: String, data: Vec<u8> },

    /// Image transfer message
    Image { name: String, data: Vec<u8> },

    /// Error message with code and description
    Error { code: ErrorCode, message: String },
}

/// Helper trait for reading/writing messages over a stream
pub trait MessageStream {
    fn write_message(&mut self, message: &Message) -> io::Result<()>;
    fn read_message(&mut self) -> io::Result<Message>;
}

impl MessageStream for TcpStream {
    fn write_message(&mut self, message: &Message) -> io::Result<()> {
        let serialized = serde_cbor::to_vec(message)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let length = serialized.len() as u32;
        self.write_all(&length.to_be_bytes())?;
        self.write_all(&serialized)?;
        self.flush()?;
        Ok(())
    }

    fn read_message(&mut self) -> io::Result<Message> {
        let mut length_buf = [0u8; 4];
        self.read_exact(&mut length_buf)?;
        let length = u32::from_be_bytes(length_buf) as usize;

        let mut buffer = vec![0; length];
        self.read_exact(&mut buffer)?;

        serde_cbor::from_slice(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }
}
