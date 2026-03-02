use uuid::Uuid;

/// Input DTO for the send message use case.
pub struct SendMessageCommand {
    pub channel_type: String,
    pub channel_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
}

/// Input DTO for the get messages use case.
pub struct GetMessagesQuery {
    pub channel_type: String,
    pub channel_id: Option<Uuid>,
    pub limit: u32,
    pub before_id: Option<Uuid>,
}
