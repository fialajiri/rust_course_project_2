use crate::encryption::{file::FileEncryption, message::MessageEncryption};
use anyhow::Result;
use std::sync::Arc;

pub struct EncryptionService {
    message_encryption: Arc<MessageEncryption>,
    file_encryption: Arc<FileEncryption>,
}

impl EncryptionService {
    pub fn new(key: &[u8]) -> Result<Self> {
        Ok(Self {
            message_encryption: Arc::new(MessageEncryption::new(key)?),
            file_encryption: Arc::new(FileEncryption::new(key)?),
        })
    }

    pub fn message(&self) -> Arc<MessageEncryption> {
        Arc::clone(&self.message_encryption)
    }

    pub fn file(&self) -> Arc<FileEncryption> {
        Arc::clone(&self.file_encryption)
    }
}
