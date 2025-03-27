//! Client service for managing chat server connections.
//!
//! This module handles client connections, including:
//! - Managing client connections and their states
//! - Handling new client connections
//! - Managing client authentication states
//! - Providing encryption services for secure communication

use crate::services::connection_service::ConnectionService;
use crate::types::{AuthState, ChatRoomConnection, Clients};
use crate::utils::db_connection::DbPool;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chat_common::encryption::EncryptionService;
use chat_common::error::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{error, info};

/// Service responsible for managing client connections in the chat server.
///
/// The `ClientService` handles:
/// - New client connections
/// - Client state management
/// - Encryption setup
/// - Connection lifecycle
pub struct ClientService {
    /// Shared map of all connected clients
    clients: Clients,
    /// Atomic counter for generating unique client IDs
    next_id: AtomicUsize,
    /// Shared database connection pool
    pool: Arc<DbPool>,
    /// Shared encryption service for secure communication
    encryption: Arc<EncryptionService>,
}

impl ClientService {
    /// Creates a new `ClientService` instance.
    ///
    /// # Arguments
    /// * `clients` - Shared map of all connected clients
    /// * `pool` - Shared database connection pool
    ///
    /// # Returns
    /// * `Result<Self>` - The new ClientService instance or an error if initialization fails
    ///
    /// # Panics
    /// * If ENCRYPTION_KEY environment variable is not set
    /// * If ENCRYPTION_KEY is not valid base64
    /// * If decoded ENCRYPTION_KEY is not exactly 32 bytes
    pub fn new(clients: Clients, pool: Arc<DbPool>) -> Result<Self> {
        let key = std::env::var("ENCRYPTION_KEY")
            .expect("ENCRYPTION_KEY environment variable must be set");

        let key_bytes = BASE64
            .decode(key)
            .expect("ENCRYPTION_KEY must be base64 encoded");

        if key_bytes.len() != 32 {
            panic!("ENCRYPTION_KEY must be exactly 32 bytes when decoded");
        }

        Ok(Self {
            clients,
            next_id: AtomicUsize::new(1),
            pool,
            encryption: Arc::new(EncryptionService::new(&key_bytes)?),
        })
    }

    /// Handles a new client connection.
    ///
    /// This method:
    /// 1. Assigns a unique ID to the client
    /// 2. Creates a new connection record
    /// 3. Spawns a new task to handle the connection
    ///
    /// # Arguments
    /// * `stream` - The TCP stream for the new client connection
    ///
    /// # Returns
    /// * `Result<()>` - Success or error handling the connection
    pub async fn handle_new_client(&self, stream: TcpStream) -> Result<()> {
        let addr = stream.peer_addr()?;
        let clients = Arc::clone(&self.clients);
        let pool = Arc::clone(&self.pool);

        let (read_half, write_half) = stream.into_split();

        let client_id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let connection = ChatRoomConnection {
            user_id: None,
            writer: write_half,
            auth_state: AuthState::NotAuthenticated,
        };

        {
            let mut clients_guard = clients.lock().await;
            clients_guard.insert(client_id, connection);
        }

        info!("New client connected: {} with ID: {}", addr, client_id);

        let mut connection_service =
            ConnectionService::new(Arc::clone(&clients), pool, Arc::clone(&self.encryption));

        tokio::spawn(async move {
            if let Err(e) = connection_service
                .handle_connection(client_id, read_half)
                .await
            {
                error!("Error handling connection from {}: {}", addr, e);
            }
        });

        Ok(())
    }
}
