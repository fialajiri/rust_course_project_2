use crate::error::{ChatError, Result};
use crate::message::Message;
use std::{fs, path::Path};

pub fn process_file_command(command: &str, path_str: &str) -> Result<Message> {
    let path = Path::new(path_str.trim());

    println!("Processing file command: {}", path_str);
    println!("Command: {}", command);

    if !path.exists() {
        return Err(ChatError::NotFound(path_str.to_string()));
    }

    if !path.is_file() {
        return Err(ChatError::InvalidInput(format!("Not a file: {}", path_str)));
    }

    let data = fs::read(path)?;
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

pub fn save_file(name: &str, data: Vec<u8>) -> Result<()> {
    let path = Path::new("files").join(name);    
    create_directory("files")?;
    fs::write(path, data)?;
    Ok(())
}

pub fn save_image(name: &str, data: Vec<u8>) -> Result<()> {
    let img = image::load_from_memory(&data)
        .map_err(|e| ChatError::ImageProcessingError(format!("Failed to process image: {}", e)))?;

    let name_without_extension = name.split('.').next().unwrap_or(name);

    let timestamp = chrono::Utc::now().timestamp();
    let path = Path::new("images").join(format!("{}_{}.png", name_without_extension, timestamp));

    create_directory("images")?;

    img.save_with_format(&path, image::ImageFormat::Png)
        .map_err(|e| ChatError::ImageProcessingError(e.to_string()))?;

    Ok(())
}

pub fn create_directory(path: &str) -> Result<()> {
    let path = Path::new(path);
    fs::create_dir_all(path)?;
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
        // This test needs to be updated to use a real image file
        // for now, we'll just test the error case
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fake image data").unwrap();

        let result = process_file_command(".image", file_path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_process_file_command_invalid() {
        let result = process_file_command(".invalid", "nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_error_message() {
        let error = ChatError::NotFound("test.txt".to_string());
        let message = create_error_message(&error);

        if let Message::Error { code, message: msg } = message {
            assert_eq!(code, crate::error::ErrorCode::FileNotFound);
            assert_eq!(msg, "File not found: test.txt");
        } else {
            panic!("Expected Error message");
        }
    }

    #[test]
    fn test_create_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test");
        assert!(create_directory(&path.to_str().unwrap()).is_ok());
        assert!(path.exists());
    }
}
