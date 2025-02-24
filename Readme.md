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

## Overview

This project consists of a chat server and client implemented in Rust. The server can handle multiple clients simultaneously, allowing them to send text messages, files, and images to each other. The client can connect to the server, send messages, and receive messages from other clients.

## Features

- **Multi-client support**: The server can handle multiple clients at the same time.
- **Message types**: Clients can send text messages, files, and images.
- **File and image handling**: Received files and images are saved in designated directories.
- **Command recognition**: The client recognizes special commands like `.file`, `.image`, and `.quit`.

## Usage

### Server

To start the server, run the following command:

cargo run --bin chat-server -- --addr &lt;hostname&gt;:&lt;port&gt;

- Replace `<hostname>` with the desired hostname or IP address.
- Replace `<port>` with the desired port number.
- If no address is provided, the server will use default values.

### Client

To start the client, run the following command:

cargo run --bin chat-client -- --addr &lt;hostname&gt;:&lt;port&gt;

- Replace `<hostname>` with the server's hostname or IP address.
- Replace `<port>` with the server's port number.
- If no address is provided, the client will use default values.

### Commands

- **Text Message**: Simply type your message and press Enter to send it.
- **File**: Use the command `.file <path>` to send a file. Replace `<path>` with the path to the file you want to send.
- **Image**: Use the command `.image <path>` to send an image. Replace `<path>` with the path to the image you want to send.
- **Quit**: Use the command `.quit` to disconnect the client from the server.

### Directories

- **Images**: Received images are saved in the `images/` directory.
- **Files**: Received files are saved in the `files/` directory.

## Dependencies

- **Clap**: For command-line argument parsing.
- **Chat Common**: A shared library for message handling and file operations.

## Building

To build the project, run:

cargo build --release

This will compile the server and client binaries in the `target/release` directory.
