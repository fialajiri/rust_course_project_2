# Rust Developer Course

## Homework X

### Completed Requirements

- **Web Frontend Development** ✓

  - Created a web frontend for server application
  - Implemented message viewing functionality
  - Added user-based message filtering
  - Built using Trunk and Nginx for serving

- **User and Message Management** ✓

  - Implemented user deletion functionality
  - Added message deletion capabilities
  - Integrated user-message relationship management
  - Ensured proper data cleanup on user deletion

- **Web Framework Selection** ✓

  - Chose Yew for frontend development
  - Integrated with existing async server backend
  - Ensured seamless communication between frontend and backend

- **Backend Integration** ✓

  - Successfully integrated frontend with async server
  - Implemented efficient data fetching and display
  - Ensured real-time updates and data consistency

- **Interface Design** ✓
  - Created intuitive and user-friendly interface
  - Implemented clean and navigable layout  

## Overview

This project consists of a chat server and client implemented in Rust. The server can handle multiple clients simultaneously, allowing them to send text messages, files, and images to each other. The client can connect to the server, send messages, and receive messages from other clients. Additionally, a web frontend is provided for administrative purposes, allowing management of users and messages.

## Features

- **Multi-client support**: The server can handle multiple clients at the same time.
- **Message types**: Clients can send text messages, files, and images.
- **File and image handling**: Received files and images are saved in designated directories.
- **Command recognition**: The client recognizes special commands like `.file`, `.image`, and `.quit`.
- **Web Administration**: A web frontend for managing users and messages.
  - User management (view, delete)
  - Message management (view, filter, delete)
  - Message filtering by user

## Usage

### Server, Database, and Frontend Setup

The server, database, and web frontend are managed using Docker Compose. To start them:

1. Make sure you have Docker and Docker Compose installed
2. Navigate to the project root directory
3. Run:

```bash
docker compose up -d
```

This will start:

- The chat server
- PostgreSQL database
- Web frontend (accessible at http://localhost:80)

### Client

To start the client, run the following command:

`cargo run --bin chat-client`

### Web Frontend

The web frontend provides an administrative interface accessible at http://localhost:80. Features include:

- Viewing all messages
- Filtering messages by user
- Deleting users and their associated messages
- Managing user accounts

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
- **Frontend Dependencies**:
  - **Trunk**: For building the web frontend
  - **Nginx**: For serving the frontend application

## Building

To build the project, run:

`cargo build --release`

This will compile the server and client binaries in the `target/release` directory.

## Testing

To run the tests, use:

`cargo test`

This will execute all the tests in the project, ensuring that the functionality is working as expected.
