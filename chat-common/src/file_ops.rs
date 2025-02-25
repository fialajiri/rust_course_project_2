use crate::Message;
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileOpsError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Failed to read file: {0}")]
    ReadError(#[from] io::Error),
    #[error("Failed to process image: {0}")]
    ImageProcessingError(String),
}

pub fn process_file_command(command: &str, path_str: &str) -> Result<Message, FileOpsError> {
    let path = Path::new(path_str.trim());

    println!("Processing file command: {}", path_str);
    println!("Command: {}", command);

    if !path.exists() {
        return Err(FileOpsError::NotFound(path_str.to_string()));
    }

    if !path.is_file() {
        return Err(FileOpsError::InvalidInput(path_str.to_string()));
    }

    let data = fs::read(path)?;
    let name = path
        .file_name()
        .ok_or_else(|| FileOpsError::InvalidInput("Invalid file name".to_string()))?
        .to_string_lossy()
        .into();

    match command {
        ".file" => Ok(Message::File { name, data }),
        ".image" => Ok(Message::Image { name, data }),
        _ => Err(FileOpsError::InvalidInput("Invalid command".to_string())),
    }
}

pub fn save_file(name: &str, data: Vec<u8>) -> Result<(), FileOpsError> {
    let path = Path::new("files").join(name);
    fs::write(path, data).map_err(FileOpsError::ReadError)
}

pub fn save_image(name: &str, data: Vec<u8>) -> Result<(), FileOpsError> {
    let img = image::load_from_memory(&data).map_err(|e| {
        FileOpsError::ImageProcessingError(format!("Failed to process image: {}", e))
    })?;

    let name_without_extension = name.split('.').next().unwrap_or(name);

    let timestamp = chrono::Utc::now().timestamp();
    let path = Path::new("images").join(format!("{}_{}.png", name_without_extension, timestamp));

    img.save_with_format(&path, image::ImageFormat::Png)
        .map_err(|e| FileOpsError::ImageProcessingError(e.to_string()))
}
