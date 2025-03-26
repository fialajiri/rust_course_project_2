use crate::encryption::{file::FileEncryption, message::MessageEncryption};
use anyhow::Result;
use std::sync::Arc;

/// A service that provides access to both message and file encryption capabilities
///
/// This service wraps both message and file encryption implementations in thread-safe
/// reference-counted containers, allowing them to be shared across threads.
pub struct EncryptionService {
    message_encryption: Arc<MessageEncryption>,
    file_encryption: Arc<FileEncryption>,
}

impl EncryptionService {
    /// Creates a new EncryptionService instance with the provided key
    ///
    /// # Arguments
    /// * `key` - A 32-byte key that will be used for both message and file encryption
    ///
    /// # Returns
    /// * `Result<Self>` - A new EncryptionService instance or an error if key initialization fails
    pub fn new(key: &[u8]) -> Result<Self> {
        Ok(Self {
            message_encryption: Arc::new(MessageEncryption::new(key)?),
            file_encryption: Arc::new(FileEncryption::new(key)?),
        })
    }

    /// Returns a thread-safe reference to the message encryption service
    ///
    /// # Returns
    /// * `Arc<MessageEncryption>` - A thread-safe reference to the message encryption service
    pub fn message(&self) -> Arc<MessageEncryption> {
        Arc::clone(&self.message_encryption)
    }

    /// Returns a thread-safe reference to the file encryption service
    ///
    /// # Returns
    /// * `Arc<FileEncryption>` - A thread-safe reference to the file encryption service
    pub fn file(&self) -> Arc<FileEncryption> {
        Arc::clone(&self.file_encryption)
    }
}
