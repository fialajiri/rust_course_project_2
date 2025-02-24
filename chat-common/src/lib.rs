use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;

pub mod file_ops;

#[derive(Serialize, Deserialize, Clone)]
pub enum Message {
    Text(String),
    File { name: String, data: Vec<u8> },
    Image { name: String, data: Vec<u8> },
}

// Helper trait for reading/writing messages
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

#[derive(Parser)]
pub struct Args {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: String,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
}

impl Args {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}


