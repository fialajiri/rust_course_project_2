use crate::services::connection_service::ConnectionService;
use crate::types::{AuthState, ChatRoomConnection, Clients};
use crate::utils::db_connection::DbPool;
use chat_common::encryption::EncryptionService;
use chat_common::error::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{error, info};

const ENCRYPTION_KEY: [u8; 32] = [0; 32]; // Replace with secure key management

pub struct ClientService {
    clients: Clients,
    next_id: AtomicUsize,
    pool: Arc<DbPool>,
    encryption: Arc<EncryptionService>,
}

impl ClientService {
    pub fn new(clients: Clients, pool: Arc<DbPool>) -> Result<Self> {
        Ok(Self {
            clients,
            next_id: AtomicUsize::new(1),
            pool,
            encryption: Arc::new(EncryptionService::new(&ENCRYPTION_KEY)?),
        })
    }

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
