use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct ChatRoomConnection {
    pub user_id: i32,
    pub writer: OwnedWriteHalf,
}

/// Type alias for the shared clients collection
pub type Clients = Arc<Mutex<HashMap<usize, ChatRoomConnection>>>;
