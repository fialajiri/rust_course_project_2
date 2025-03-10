use crate::{Message, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

#[async_trait::async_trait]
pub trait AsyncMessageStream {
    async fn read_message(&mut self) -> Result<Message>;
    async fn write_message(&mut self, message: &Message) -> Result<()>;
}

#[async_trait::async_trait]
impl AsyncMessageStream for TcpStream {
    async fn read_message(&mut self) -> Result<Message> {
        let mut len_bytes = [0u8; 4];
        self.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        self.read_exact(&mut buffer).await?;

        Ok(serde_cbor::from_slice(&buffer)?)
    }

    async fn write_message(&mut self, message: &Message) -> Result<()> {
        let bytes = serde_cbor::to_vec(message)?;
        self.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
        self.write_all(&bytes).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl AsyncMessageStream for OwnedReadHalf {
    async fn read_message(&mut self) -> Result<Message> {
        let mut len_bytes = [0u8; 4];
        self.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        self.read_exact(&mut buffer).await?;

        Ok(serde_cbor::from_slice(&buffer)?)
    }

    async fn write_message(&mut self, _message: &Message) -> Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Cannot write messages with ReadHalf",
        )
        .into())
    }
}

#[async_trait::async_trait]
impl AsyncMessageStream for OwnedWriteHalf {
    async fn read_message(&mut self) -> Result<Message> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Cannot read messages with WriteHalf",
        )
        .into())
    }

    async fn write_message(&mut self, message: &Message) -> Result<()> {
        let bytes = serde_cbor::to_vec(message)?;
        self.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
        self.write_all(&bytes).await?;
        Ok(())
    }
}
