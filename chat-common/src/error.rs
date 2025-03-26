use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

/// Error codes that can be returned by the chat application
///
/// These codes provide a high-level categorization of errors that can occur
/// during chat operations. They are designed to be serializable and can be
/// transmitted over the network.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    /// File requested was not found
    FileNotFound,
    /// Operation was denied due to insufficient permissions
    PermissionDenied,
    /// Input provided was invalid or malformed
    InvalidInput,
    /// An error occurred on the server side
    ServerError,
    /// A network-related error occurred
    NetworkError,
    /// An error occurred while processing an image
    ImageProcessingError,
    /// An unknown or unexpected error occurred
    UnknownError,
}

/// Detailed error types that can occur in the chat application
///
/// This enum provides specific error information with descriptive messages.
/// It implements the `Error` trait and can be converted to `ErrorCode` for
/// network transmission.
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

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),
}

impl ChatError {
    /// Converts a detailed error into a network-transmittable error code
    ///
    /// # Returns
    /// * `ErrorCode` - The corresponding error code for network transmission
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
            ChatError::InvalidPath(_) => ErrorCode::UnknownError,
            ChatError::InvalidCommand(_) => ErrorCode::UnknownError,
        }
    }
}

impl From<serde_cbor::Error> for ChatError {
    fn from(err: serde_cbor::Error) -> Self {
        ChatError::SerializationError(err.to_string())
    }
}

impl From<anyhow::Error> for ChatError {
    fn from(err: anyhow::Error) -> Self {
        ChatError::UnknownError(err.to_string())
    }
}

impl From<serde_json::Error> for ChatError {
    fn from(err: serde_json::Error) -> Self {
        ChatError::SerializationError(err.to_string())
    }
}

/// A type alias for Result using ChatError as the error type
pub type Result<T> = std::result::Result<T, ChatError>;
