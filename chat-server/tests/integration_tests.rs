use chat_common::{Message, MessageStream};
use chat_server::message_handler::MessageHandler;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn setup_test_server() -> (std::net::SocketAddr, Arc<Mutex<Vec<TcpStream>>>) {
    let clients = Arc::new(Mutex::new(Vec::new()));
    let server_clients = clients.clone();
    let handler = MessageHandler::new(clients.clone());

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let server_addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                clients.lock().unwrap().push(stream.try_clone().unwrap());
                let mut stream_clone = stream.try_clone().unwrap();
                let handler_clone = handler.clone();
                thread::spawn(move || {
                    while let Ok(message) = stream_clone.read_message() {
                        if let Err(e) = handler_clone.process_message(&stream_clone, &message) {
                            eprintln!("Error processing message: {}", e);
                            break;
                        }
                    }
                    let _ = handler_clone.handle_disconnect(&stream_clone);
                });
            }
        }
    });

    // Give the server a moment to start
    thread::sleep(Duration::from_millis(50));

    (server_addr, server_clients)
}

#[test]
fn test_client_connection() {
    let (server_addr, server_clients) = setup_test_server();

    // Connect to the server
    let _client = TcpStream::connect(server_addr).unwrap();

    // Give the server a moment to process the connection
    thread::sleep(Duration::from_millis(100));

    assert_eq!(server_clients.lock().unwrap().len(), 1);
}

#[test]
fn test_message_broadcast() {
    let (server_addr, _) = setup_test_server();

    // Create two client connections
    let mut client1 = TcpStream::connect(server_addr).unwrap();
    thread::sleep(Duration::from_millis(100));

    let mut client2 = TcpStream::connect(server_addr).unwrap();
    thread::sleep(Duration::from_millis(100));

    // Client 1 sends a message
    let test_message = Message::Text("Hello from client 1".to_string());
    client1.write_message(&test_message).unwrap();

    // Give some time for the message to be processed
    thread::sleep(Duration::from_millis(100));

    // Client 2 should receive the message
    if let Ok(received_message) = client2.read_message() {
        match received_message {
            Message::Text(content) => {
                assert_eq!(content, "Hello from client 1");
            }
            _ => panic!("Unexpected message type"),
        }
    } else {
        panic!("Failed to receive message");
    }
}

#[test]
fn test_client_disconnect() {
    let (server_addr, server_clients) = setup_test_server();

    // Connect a client
    let client = TcpStream::connect(server_addr).unwrap();
    thread::sleep(Duration::from_millis(100));

    assert_eq!(server_clients.lock().unwrap().len(), 1);

    // Disconnect the client
    drop(client);
    thread::sleep(Duration::from_millis(100));

    // The server should remove the disconnected client
    assert_eq!(server_clients.lock().unwrap().len(), 0);
}

#[test]
fn test_file_transfer() {
    let (server_addr, _) = setup_test_server();

    // Create two client connections
    let mut sender = TcpStream::connect(server_addr).unwrap();
    thread::sleep(Duration::from_millis(100));

    let mut receiver = TcpStream::connect(server_addr).unwrap();
    thread::sleep(Duration::from_millis(100));

    // Create a test file message
    let file_content = vec![1, 2, 3, 4, 5];
    let file_message = Message::File {
        name: "test.txt".to_string(),
        data: file_content.clone(),
    };

    // Send the file
    sender.write_message(&file_message).unwrap();
    thread::sleep(Duration::from_millis(100));

    // Verify the receiver gets the file
    if let Ok(received_message) = receiver.read_message() {
        match received_message {
            Message::File { name, data } => {
                assert_eq!(name, "test.txt");
                assert_eq!(data, file_content);
            }
            _ => panic!("Unexpected message type"),
        }
    } else {
        panic!("Failed to receive file");
    }
}
