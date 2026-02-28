use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub channel: ChatChannel,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatChannel {
    Global,
    Group { group_id: Uuid },
    Whisper { to_user_id: Uuid },
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub channel: ChatChannel,
    pub content: String,
}
