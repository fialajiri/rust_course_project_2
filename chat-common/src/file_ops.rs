use crate::Message;
use std::{fs, io, path::Path};

pub fn process_file_command(command: &str, path_str: &str) -> io::Result<Message> {
    let path = Path::new(path_str.trim());

    println!("Processing file command: {}", path_str);
    println!("Command: {}", command);

    
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", path_str),
        ));
    }
    
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Not a file: {}", path_str),
        ));
    }

    let data = fs::read(path)?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?
        .to_string_lossy()
        .into();

    match command {
        ".file" => Ok(Message::File { name, data }),
        ".image" => Ok(Message::Image { name, data }),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid command",
        )),
    }
}

pub fn save_file(name: &str, data: Vec<u8>) -> io::Result<()> {
    let path = Path::new("files").join(name);
    fs::write(path, data)
}

pub fn save_image(name: &str, data: Vec<u8>) -> io::Result<()> {    
    let img = image::load_from_memory(&data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to process image: {}", e),
        )
    })?;

    let name_without_extension = name.split('.').next().unwrap_or(name);

    let timestamp = chrono::Utc::now().timestamp();
    let path = Path::new("images").join(format!("{}_{}.png", name_without_extension, timestamp));

    img.save_with_format(&path, image::ImageFormat::Png)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}


