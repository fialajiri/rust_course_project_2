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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_process_file_command_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let result = process_file_command(".file", file_path.to_str().unwrap());
        assert!(result.is_ok());
        if let Ok(Message::File { name, data }) = result {
            assert_eq!(name, "test.txt");
            assert_eq!(data, b"Hello, world!\n");
        }
    }

    #[test]
    fn test_process_file_command_image() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fake image data").unwrap();

        let result = process_file_command(".image", file_path.to_str().unwrap());
        assert!(result.is_ok());
        if let Ok(Message::Image { name, data }) = result {
            assert_eq!(name, "test.png");
            assert_eq!(data, b"fake image data\n");
        }
    }

    #[test]
    fn test_process_file_command_invalid() {
        let result = process_file_command(".invalid", "nonexistent.txt");
        assert!(result.is_err());
    }
}
