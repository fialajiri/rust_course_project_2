use clap::Parser;

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;

pub mod async_message_stream;
pub mod error;
pub mod file_ops;

// Re-export commonly used items
pub use async_message_stream::AsyncMessageStream;
pub use error::{ChatError, ErrorCode, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Message {
    Text(String),
    System(String),
    File { name: String, data: Vec<u8> },
    Image { name: String, data: Vec<u8> },
    Error { code: ErrorCode, message: String },
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn test_message_stream_write_and_read() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let message = Message::Text("Hello, world!".to_string());
            AsyncMessageStream::write_message(&mut stream, &message)
                .await
                .unwrap();
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let message = AsyncMessageStream::read_message(&mut stream).await.unwrap();
        assert_eq!(message, Message::Text("Hello, world!".to_string()));

        server.await.unwrap();
    }
}
