use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Size of chunks used for file encryption/decryption operations
const CHUNK_SIZE: usize = 1024 * 64; // 64KB chunks

/// Metadata required for file decryption
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedFileMetadata {
    /// Base64 encoded nonce used for encryption
    pub nonce: String,
    /// Original size of the file before encryption
    pub original_size: u64,
}

/// Handles file encryption and decryption using AES-256-GCM
pub struct FileEncryption {
    cipher: Aes256Gcm,
}

impl FileEncryption {
    /// Creates a new FileEncryption instance with the provided key
    ///
    /// # Arguments
    /// * `key` - A 32-byte key for AES-256-GCM encryption
    ///
    /// # Returns
    /// * `Result<Self>` - A new FileEncryption instance or an error if the key length is invalid
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(anyhow!("Key must be exactly 32 bytes"));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    /// Encrypts a file stream using AES-256-GCM
    ///
    /// # Arguments
    /// * `reader` - Async reader providing the input data
    /// * `writer` - Async writer for the encrypted output
    ///
    /// # Returns
    /// * `Result<EncryptedFileMetadata>` - Metadata required for decryption or an error if encryption fails
    pub async fn encrypt_stream<R, W>(
        &self,
        mut reader: R,
        mut writer: W,
    ) -> Result<EncryptedFileMetadata>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut total_size = 0u64;
        let mut buffer = vec![0u8; CHUNK_SIZE];

        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            total_size += n as u64;

            let encrypted = self
                .cipher
                .encrypt(nonce, &buffer[..n])
                .map_err(|e| anyhow!("Encryption failed: {}", e))?;

            writer.write_all(&encrypted).await?;
        }

        writer.flush().await?;

        Ok(EncryptedFileMetadata {
            nonce: BASE64.encode(nonce_bytes),
            original_size: total_size,
        })
    }

    /// Decrypts a file stream using AES-256-GCM
    ///
    /// # Arguments
    /// * `reader` - Async reader providing the encrypted data
    /// * `writer` - Async writer for the decrypted output
    /// * `metadata` - Metadata containing the nonce and original file size
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if decryption fails
    pub async fn decrypt_stream<R, W>(
        &self,
        mut reader: R,
        mut writer: W,
        metadata: &EncryptedFileMetadata,
    ) -> Result<()>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let nonce_bytes = BASE64
            .decode(&metadata.nonce)
            .map_err(|e| anyhow!("Invalid base64 nonce: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut buffer = vec![0u8; CHUNK_SIZE + 16];
        let mut bytes_remaining = metadata.original_size;

        while bytes_remaining > 0 {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let decrypted = self
                .cipher
                .decrypt(nonce, &buffer[..n])
                .map_err(|e| anyhow!("Decryption failed: {}", e))?;

            writer.write_all(&decrypted).await?;
            bytes_remaining -= decrypted.len() as u64;
        }

        writer.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn test_file_encryption_decryption() {
        let key = [0u8; 32];
        let encryption = FileEncryption::new(&key).unwrap();

        let original_data = b"Hello, World!";
        let mut encrypted = Vec::new();

        let metadata = encryption
            .encrypt_stream(BufReader::new(&original_data[..]), &mut encrypted)
            .await
            .unwrap();

        let mut decrypted = Vec::new();
        encryption
            .decrypt_stream(BufReader::new(&encrypted[..]), &mut decrypted, &metadata)
            .await
            .unwrap();

        assert_eq!(&original_data[..], &decrypted[..]);
    }
}
