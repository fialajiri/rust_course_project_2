use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    File,
    Image,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: i32,
    pub sender_id: i32,
    pub message_type: MessageType,
    pub content: Option<String>,
    pub file_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
