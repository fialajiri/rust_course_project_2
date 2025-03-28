use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

/// Represents an encrypted message with its associated metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Base64 encoded encrypted data
    pub ciphertext: String,
    /// Base64 encoded nonce used for encryption
    pub nonce: String,
}

/// Handles message encryption and decryption using AES-256-GCM
pub struct MessageEncryption {
    cipher: Aes256Gcm,
}

impl MessageEncryption {
    /// Creates a new MessageEncryption instance with the provided key
    ///
    /// # Arguments
    /// * `key` - A 32-byte key for AES-256-GCM encryption
    ///
    /// # Returns
    /// * `Result<Self>` - A new MessageEncryption instance or an error if the key length is invalid
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(anyhow!("Key must be exactly 32 bytes"));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    /// Generates a new random encryption key suitable for AES-256-GCM
    ///
    /// # Returns
    /// * `[u8; 32]` - A 32-byte array containing the randomly generated key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypts a message using AES-256-GCM
    ///
    /// # Arguments
    /// * `message` - The plaintext message to encrypt
    ///
    /// # Returns
    /// * `Result<EncryptedMessage>` - The encrypted message with its metadata or an error if encryption fails
    pub fn encrypt(&self, message: &str) -> Result<EncryptedMessage> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, message.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedMessage {
            ciphertext: BASE64.encode(ciphertext),
            nonce: BASE64.encode(nonce_bytes),
        })
    }

    /// Decrypts a message using AES-256-GCM
    ///
    /// # Arguments
    /// * `encrypted` - The encrypted message with its metadata
    ///
    /// # Returns
    /// * `Result<String>` - The decrypted plaintext message or an error if decryption fails
    pub fn decrypt(&self, encrypted: &EncryptedMessage) -> Result<String> {
        let ciphertext = BASE64
            .decode(&encrypted.ciphertext)
            .map_err(|e| anyhow!("Invalid base64 ciphertext: {}", e))?;

        let nonce_bytes = BASE64
            .decode(&encrypted.nonce)
            .map_err(|e| anyhow!("Invalid base64 nonce: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = MessageEncryption::generate_key();
        let encryption = MessageEncryption::new(&key).unwrap();

        let original = "Hello, World!";
        let encrypted = encryption.encrypt(original).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }
}
