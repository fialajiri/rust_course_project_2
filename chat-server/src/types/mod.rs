use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ChatRoomConnection {
    pub user_id: Option<i32>,
    pub writer: OwnedWriteHalf,
    pub auth_state: AuthState,
}

/// Type alias for the shared clients collection
pub type Clients = Arc<Mutex<HashMap<usize, ChatRoomConnection>>>;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    NotAuthenticated,
    Authenticated { user_id: i32, token: String },
}

impl ChatRoomConnection {
    pub fn is_authenticated(&self) -> bool {
        matches!(self.auth_state, AuthState::Authenticated { .. })
    }
}
