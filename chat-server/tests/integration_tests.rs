// use chat_common::{async_message_stream::AsyncMessageStream, Message};
// use chat_server::message_handler::MessageHandler;
// use std::sync::Arc;
// use std::time::Duration;
// use tokio::net::{
//     tcp::{OwnedReadHalf, OwnedWriteHalf},
//     TcpStream,
// };
// use tokio::sync::Mutex;

// async fn setup_test_server() -> (std::net::SocketAddr, Arc<Mutex<Vec<OwnedWriteHalf>>>) {
//     let clients = Arc::new(Mutex::new(Vec::new()));
//     let server_clients = clients.clone();
//     let handler = MessageHandler::new(clients.clone());

//     let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
//     let server_addr = listener.local_addr().unwrap();

//     let server_clients_clone = server_clients.clone();
//     tokio::spawn(async move {
//         loop {
//             if let Ok((stream, _)) = listener.accept().await {
//                 let (mut read_half, write_half) = stream.into_split();
//                 server_clients_clone.lock().await.push(write_half);

//                 let handler_clone = handler.clone();
//                 tokio::spawn(async move {
//                     while let Ok(message) = read_half.read_message().await {
//                         if let Err(e) = handler_clone
//                             .process_message::<OwnedReadHalf>(None, &message)
//                             .await
//                         {
//                             eprintln!("Error processing message: {}", e);
//                             break;
//                         }
//                     }
//                     let _ = handler_clone.handle_disconnect(read_half).await;
//                 });
//             }
//         }
//     });

//     // Give the server a moment to start
//     tokio::time::sleep(Duration::from_millis(50)).await;

//     (server_addr, server_clients)
// }

// #[tokio::test]
// async fn test_client_connection() {
//     let (server_addr, server_clients) = setup_test_server().await;

//     // Connect to the server
//     let _client = TcpStream::connect(server_addr).await.unwrap();

//     // Give the server a moment to process the connection
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     assert_eq!(server_clients.lock().await.len(), 1);
// }

// #[tokio::test]
// async fn test_message_broadcast() {
//     let (server_addr, server_clients) = setup_test_server().await;

//     // Create two client connections
//     let client1 = TcpStream::connect(server_addr).await.unwrap();
//     let (mut read1, mut write1) = client1.into_split();

//     // Create server connection for client1
//     let client1_for_server = TcpStream::connect(server_addr).await.unwrap();
//     let (_, write1_for_server) = client1_for_server.into_split();
//     server_clients.lock().await.push(write1_for_server);
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     let client2 = TcpStream::connect(server_addr).await.unwrap();
//     let (mut read2, write2) = client2.into_split();
//     server_clients.lock().await.push(write2);
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // Client 1 sends a message
//     let test_message = Message::Text("Hello from client 1".to_string());
//     write1.write_message(&test_message).await.unwrap();

//     // Give some time for the message to be processed
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // Client 2 should receive the message
//     if let Ok(received_message) = read2.read_message().await {
//         match received_message {
//             Message::Text(content) => {
//                 assert_eq!(content, "Hello from client 1");
//             }
//             _ => panic!("Unexpected message type"),
//         }
//     } else {
//         panic!("Failed to receive message");
//     }

//     // Client 1 should receive system acknowledgment
//     if let Ok(ack) = read1.read_message().await {
//         match ack {
//             Message::System(content) => {
//                 assert!(content.contains("successfully"));
//             }
//             _ => panic!("Expected system acknowledgment"),
//         }
//     }
// }

// #[tokio::test]
// async fn test_client_disconnect() {
//     let (server_addr, server_clients) = setup_test_server().await;

//     // Connect a client
//     let client = TcpStream::connect(server_addr).await.unwrap();
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     assert_eq!(server_clients.lock().await.len(), 1);

//     // Disconnect the client
//     drop(client);
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // The server should remove the disconnected client
//     assert_eq!(server_clients.lock().await.len(), 0);
// }

// #[tokio::test]
// async fn test_file_transfer() {
//     let (server_addr, server_clients) = setup_test_server().await;

//     // Create two client connections
//     let sender = TcpStream::connect(server_addr).await.unwrap();
//     let (mut read_sender, mut write_sender) = sender.into_split();

//     // Create another connection for the server's client list
//     let sender_for_server = TcpStream::connect(server_addr).await.unwrap();
//     let (_, write_sender_for_server) = sender_for_server.into_split();
//     server_clients.lock().await.push(write_sender_for_server);
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     let receiver = TcpStream::connect(server_addr).await.unwrap();
//     let (mut read_receiver, write_receiver) = receiver.into_split();
//     server_clients.lock().await.push(write_receiver);
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // Create a test file message
//     let file_content = vec![1, 2, 3, 4, 5];
//     let file_message = Message::File {
//         name: "test.txt".to_string(),
//         data: file_content.clone(),
//     };

//     // Send the file
//     write_sender.write_message(&file_message).await.unwrap();
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // Verify the receiver gets the file
//     if let Ok(received_message) = read_receiver.read_message().await {
//         match received_message {
//             Message::File { name, data } => {
//                 assert_eq!(name, "test.txt");
//                 assert_eq!(data, file_content);
//             }
//             _ => panic!("Unexpected message type"),
//         }
//     } else {
//         panic!("Failed to receive file");
//     }

//     // Sender should receive system acknowledgment
//     if let Ok(ack) = read_sender.read_message().await {
//         match ack {
//             Message::System(content) => {
//                 assert!(content.contains("successfully"));
//             }
//             _ => panic!("Expected system acknowledgment"),
//         }
//     }
// }
