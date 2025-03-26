<!-- @format -->

# Rust Developer Course

## Homework V

- Design the server to receive messages from multiple clients ✓
- Accept port and hostname as parameters ✓
- If none are provided, set default port and hostname ✓
- Clients should connect to server and send messages ✓
- Clients should accept port and hostname parameters ✓
- Clients should read from stdin and recognize distinct messages ✓
- Recognized message should be .image and .file ✓
- The .quit command terminates the client ✓
- When the client receives images, save them in the images/ directory, naming them &lt;timestamp&gt;.png. ✓
- Here I improve the solution and kept the original image name and to &lt;original_name_timestamp&gt;.png. ✓
- Automatically convert any receiving images into png type ✓

## Homework VI

- Transform both client and server parts into separate Cargo crates ✓
- Structure the project directory to clearly separate the two parts of the application ✓
- Identify the shared functionality and abstract into the common library ✓
- Added `tracing` and `tracing-subscriber` crates for logging ✓
- Added `chrono` crate for dates and time operations ✓
- Added `anyhow` for better error handling and edding contect to errors ✓
- Added `thiserror` crate for creating custom errors ✓
- Added `tempfile` crate for managing temprary files and directories ✓
- Updated the `README.md` with documentation of how to use the applications ✓
- Added comments to the code to explain reasoning ✓
- Added test for client, server and common library ✓

## Homework VII

### Error Handling Improvements

- Enhanced error handling using `anyhow` and `thiserror` crates ✓
  - Custom error types for specific failure scenarios
  - Detailed error context and chain tracking
  - Better error messages for debugging
- Implemented comprehensive server-side error handling ✓
  - Graceful handling of connection failures
  - Detailed logging of error conditions
- Added client-side error management ✓
  - New message types for Error and System notifications
  - User-friendly error messages

### Code Structure Improvements

- Refactored server architecture ✓
  - Split into logical modules (connection, message handling, state management)
  - Improved separation of concerns
  - Better resource management
- Reorganized client codebase ✓
  - Modular command processing
  - Separate UI and network layers
  - Cleaner message handling logic

### Testing Enhancements

- Expanded test coverage ✓
  - Unit tests for core functionality
  - Integration tests for client-server communication
  - Error handling test cases

## Homework VIII

### Asynchronous Rewriting Using Tokio

- Refactored both client and server side to use `tokio` ✓
- All I/O operations, network communications and other tasks are handled by Tokio ✓

### Database Integration

- Added Postgres db using `diesel` database framework ✓
- Chat messages are now stored in db ✓
- Setup migration for the db ✓

### User Authentication

- Added `.login` command for user identification and authentication ✓
- Pre-configured test users available for testing ✓
- Secure password hashing using bcrypt ✓

### Security Considerations

- Added encryption service to encrypt user messages, files and images ✓
- Secure password storage with bcrypt hashing ✓
- Database connection security ✓

### Refactoring

- Both client and server were refactored extensively and code is now better organized ✓
- Docker Compose setup for easy server and database deployment ✓

## Overview

This project consists of a chat server and client implemented in Rust. The server can handle multiple clients simultaneously, allowing them to send text messages, files, and images to each other. The client can connect to the server, send messages, and receive messages from other clients.

## Features

- **Multi-client support**: The server can handle multiple clients at the same time.
- **Message types**: Clients can send text messages, files, and images.
- **File and image handling**: Received files and images are saved in designated directories.
- **Command recognition**: The client recognizes special commands like `.file`, `.image`, and `.quit`.

## Usage

### Server and Database Setup

The server and database are now managed using Docker Compose. To start them:

1. Make sure you have Docker and Docker Compose installed
2. Navigate to the project root directory
3. Run:

```bash
docker compose up -d
```

This will start both the chat server and PostgreSQL database in containers.

### Client

To start the client, run the following command:

`cargo run --bin chat-client`

### Authentication

Before sending messages, you must authenticate using the `.login` command:

`.login <username> <password>`

Available test users:

- Username: `alice`, Password: `password123`
- Username: `bob`, Password: `password123`
- Username: `carol`, Password: `password123`

### Commands

- **Login**: Use `.login <username> <password>` to authenticate
- **Text Message**: Simply type your message and press Enter to send it
- **File**: Use the command `.file <path>` to send a file. Replace `<path>` with the path to the file you want to send
- **Image**: Use the command `.image <path>` to send an image. Replace `<path>` with the path to the image you want to send
- **Quit**: Use the command `.quit` to disconnect the client from the server

### Directories

- **Images**: Received images are saved in the `images/` directory
- **Files**: Received files are saved in the `files/` directory

## Dependencies

- **anyhow**: For better error handling and adding context to errors
- **base64**: For encoding and decoding binary data
- **bcrypt**: For secure password hashing
- **chrono**: For date and time operations
- **clap**: For command-line argument parsing
- **diesel**: For database operations and migrations
- **diesel-async**: For asynchronous database operations
- **dotenvy**: For loading environment variables
- **image**: For handling image files
- **rand**: For generating random values
- **serde**: For serialization and deserialization
- **serde_json**: For JSON serialization/deserialization
- **serde_cbor**: For CBOR (Concise Binary Object Representation) serialization
- **thiserror**: For creating custom errors
- **tempfile**: For managing temporary files and directories
- **tokio**: For asynchronous runtime and networking
- **tracing**: For application-level tracing and logging
- **tracing-subscriber**: For collecting and recording tracing data
- **Chat Common**: A shared library for message handling and file operations

## Building

To build the project, run:

`cargo build --release`

This will compile the server and client binaries in the `target/release` directory.

## Testing

To run the tests, use:

`cargo test`

This will execute all the tests in the project, ensuring that the functionality is working as expected.
