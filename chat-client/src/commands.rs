use anyhow::Result;
use chat_common::{file_ops, Message};
use tracing::{error, warn};

pub enum Command {
    Text(String),
    File(String),
    Image(String),
    Quit,
    Invalid,
}

pub fn parse_command(input: &str) -> Command {
    if input == ".quit" {
        return Command::Quit;
    }

    if input.starts_with(".file ") {
        let path = input.trim_start_matches(".file ").trim();
        if path.is_empty() {
            return Command::Invalid;
        }
        return Command::File(path.to_string());
    }

    if input.starts_with(".image ") {
        let path = input.trim_start_matches(".image ").trim();
        if path.is_empty() {
            return Command::Invalid;
        }
        return Command::Image(path.to_string());
    }

    if input.starts_with('.') {
        return Command::Invalid;
    }

    Command::Text(input.to_string())
}

pub fn process_command(command: Command) -> Result<Option<Message>> {
    match command {
        Command::Text(text) => Ok(Some(Message::Text(text))),
        Command::File(path) => process_file_command(".file", &path),
        Command::Image(path) => process_file_command(".image", &path),
        Command::Quit => Ok(None),
        Command::Invalid => {
            warn!("Invalid command format. Use: .file <path> or .image <path>");
            Ok(None)
        }
    }
}

fn process_file_command(command: &str, path: &str) -> Result<Option<Message>> {
    match file_ops::process_file_command(command, path) {
        Ok(msg) => Ok(Some(msg)),
        Err(e) => {
            error!("{}", e);
            Ok(Some(file_ops::create_error_message(&e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chat_common::{error::ErrorCode, Message};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_command_text() {
        let line = "Hello, world!";
        let command = parse_command(line);
        if let Command::Text(text) = command {
            assert_eq!(text, line);
        } else {
            panic!("Expected Text command");
        }
    }

    #[test]
    fn test_parse_command_file() {
        let line = ".file /path/to/file.txt";
        let command = parse_command(line);
        if let Command::File(path) = command {
            assert_eq!(path, "/path/to/file.txt");
        } else {
            panic!("Expected File command");
        }
    }

    #[test]
    fn test_parse_command_image() {
        let line = ".image /path/to/image.png";
        let command = parse_command(line);
        if let Command::Image(path) = command {
            assert_eq!(path, "/path/to/image.png");
        } else {
            panic!("Expected Image command");
        }
    }

    #[test]
    fn test_parse_command_quit() {
        let line = ".quit";
        let command = parse_command(line);
        assert!(matches!(command, Command::Quit));
    }

    #[test]
    fn test_parse_command_invalid() {
        let line = ".invalid command";
        let command = parse_command(line);
        assert!(matches!(command, Command::Invalid));
    }

    #[test]
    fn test_parse_command_empty_path() {
        let line = ".file ";
        let command = parse_command(line);
        assert!(matches!(command, Command::Invalid));
    }

    #[test]
    fn test_command_text() {
        let line = "Hello, world!";
        let command = parse_command(line);
        assert!(matches!(command, Command::Text(_)));

        let result = process_command(command).unwrap();
        assert_eq!(result, Some(Message::Text(line.to_string())));
    }

    #[test]
    fn test_command_file() {
        // Create a temporary file for testing
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Test content").unwrap();

        let line = format!(".file {}", file_path.to_string_lossy());
        let command = parse_command(&line);
        assert!(matches!(command, Command::File(_)));

        let result = process_command(command).unwrap();
        match result {
            Some(Message::File { name, data }) => {
                assert_eq!(name, "test.txt");
                assert_eq!(data, b"Test content\n");
            }
            _ => panic!("Expected File message"),
        }
    }

    #[test]
    fn test_command_image_with_invalid_image() {
        // Create a temporary file that's not a valid image
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("fake.png");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Not a real image").unwrap();

        let line = format!(".image {}", file_path.to_string_lossy());
        let command = parse_command(&line);
        assert!(matches!(command, Command::Image(_)));

        let result = process_command(command).unwrap();
        match result {
            Some(Message::Error { code, .. }) => {
                assert_eq!(code, ErrorCode::ImageProcessingError);
            }
            _ => panic!("Expected Error message for invalid image"),
        }
    }

    #[test]
    fn test_command_nonexistent_file() {
        let line = ".file /path/to/nonexistent/file.txt";
        let command = parse_command(line);
        assert!(matches!(command, Command::File(_)));

        let result = process_command(command).unwrap();
        match result {
            Some(Message::Error { code, .. }) => {
                assert_eq!(code, ErrorCode::FileNotFound);
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_command_invalid() {
        let line = ".invalid path/to/file.txt";
        let command = parse_command(line);
        assert!(matches!(command, Command::Invalid));

        let result = process_command(command).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_command_quit() {
        let line = ".quit";
        let command = parse_command(line);
        assert!(matches!(command, Command::Quit));

        let result = process_command(command).unwrap();
        assert_eq!(result, None);
    }
}
