use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    FileNotFound,
    PermissionDenied,
    InvalidInput,
    ServerError,
    NetworkError,
    ImageProcessingError,
    UnknownError,
}

#[derive(Error, Debug)]
pub enum ChatError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Image processing error: {0}")]
    ImageProcessingError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl ChatError {
    pub fn to_error_code(&self) -> ErrorCode {
        match self {
            ChatError::NotFound(_) => ErrorCode::FileNotFound,
            ChatError::InvalidInput(_) => ErrorCode::InvalidInput,
            ChatError::PermissionDenied(_) => ErrorCode::PermissionDenied,
            ChatError::ServerError(_) => ErrorCode::ServerError,
            ChatError::NetworkError(_) => ErrorCode::NetworkError,
            ChatError::ImageProcessingError(_) => ErrorCode::ImageProcessingError,
            ChatError::UnknownError(_) | ChatError::IoError(_) => ErrorCode::UnknownError,
            ChatError::SerializationError(_) => ErrorCode::UnknownError,
        }
    }
}

impl From<serde_cbor::Error> for ChatError {
    fn from(err: serde_cbor::Error) -> Self {
        ChatError::SerializationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ChatError>;
