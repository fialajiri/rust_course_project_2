use clap::Parser;

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;

pub mod error;
pub mod file_ops;
pub mod message;

// Re-export commonly used items
pub use error::{ChatError, ErrorCode, Result};
pub use message::{Message, MessageStream};

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
    use crate::message::Message;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    #[test]
    fn test_message_stream_write_and_read() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let message = Message::Text("Hello, world!".to_string());
            stream.write_message(&message).unwrap();
        });

        let mut stream = TcpStream::connect(addr).unwrap();
        let message = stream.read_message().unwrap();
        assert_eq!(message, Message::Text("Hello, world!".to_string()));

        handle.join().unwrap();
    }
}
