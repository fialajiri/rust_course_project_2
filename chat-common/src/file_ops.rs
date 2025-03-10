use crate::error::{ChatError, Result};
use crate::Message;
use std::path::Path;
use tokio::fs;

pub async fn process_file_command(command: &str, path_str: &str) -> Result<Message> {
    let path = Path::new(path_str.trim());

    println!("Processing file command: {}", path_str);
    println!("Command: {}", command);

    if !path.exists() {
        return Err(ChatError::NotFound(path_str.to_string()));
    }

    if !path.is_file() {
        return Err(ChatError::InvalidInput(format!("Not a file: {}", path_str)));
    }

    let data = fs::read(path).await?;
    let name = path
        .file_name()
        .ok_or_else(|| ChatError::InvalidInput("Invalid file name".to_string()))?
        .to_string_lossy()
        .into();

    match command {
        ".file" => Ok(Message::File { name, data }),
        ".image" => {
            if let Err(e) = image::load_from_memory(&data) {
                return Err(ChatError::ImageProcessingError(format!(
                    "Invalid image format: {}",
                    e
                )));
            }
            Ok(Message::Image { name, data })
        }
        _ => Err(ChatError::InvalidInput("Invalid command".to_string())),
    }
}

pub async fn save_file(name: &str, data: Vec<u8>) -> Result<()> {
    let path = Path::new("files").join(name);
    create_directory("files").await?;
    fs::write(path, data).await?;
    Ok(())
}

pub async fn save_image(name: &str, data: Vec<u8>) -> Result<()> {
    let img = image::load_from_memory(&data)
        .map_err(|e| ChatError::ImageProcessingError(format!("Failed to process image: {}", e)))?;

    let name_without_extension = name.split('.').next().unwrap_or(name);

    let timestamp = chrono::Utc::now().timestamp();
    let path = Path::new("images").join(format!("{}_{}.png", name_without_extension, timestamp));

    create_directory("images").await?;

    tokio::task::spawn_blocking(move || {
        img.save_with_format(&path, image::ImageFormat::Png)
            .map_err(|e| ChatError::ImageProcessingError(e.to_string()))
    })
    .await
    .unwrap()?;

    Ok(())
}

pub async fn create_directory(path: &str) -> Result<()> {
    let path = Path::new(path);
    fs::create_dir_all(path).await?;
    Ok(())
}

pub fn create_error_message(error: &ChatError) -> Message {
    Message::Error {
        code: error.to_error_code(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_file_command_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!\n").await.unwrap();

        let result = process_file_command(".file", file_path.to_str().unwrap()).await;
        assert!(result.is_ok());
        if let Ok(Message::File { name, data }) = result {
            assert_eq!(name, "test.txt");
            assert_eq!(data, b"Hello, world!\n");
        }
    }

    #[tokio::test]
    async fn test_process_file_command_image() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        fs::write(&file_path, "fake image data").await.unwrap();

        let result = process_file_command(".image", file_path.to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_file_command_invalid() {
        let result = process_file_command(".invalid", "nonexistent.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_error_message() {
        let error = ChatError::NotFound("test.txt".to_string());
        let message = create_error_message(&error);

        if let Message::Error { code, message: msg } = message {
            assert_eq!(code, crate::error::ErrorCode::FileNotFound);
            assert_eq!(msg, "File not found: test.txt");
        } else {
            panic!("Expected Error message");
        }
    }

    #[tokio::test]
    async fn test_create_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test");
        assert!(create_directory(path.to_str().unwrap()).await.is_ok());
        assert!(path.exists());
    }
}
