use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain data returned by the message repository
// ---------------------------------------------------------------------------

/// Minimal message row needed by the use cases.
pub struct MessageRow {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub channel_type: String,
    pub channel_id: Option<Uuid>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Outbound port: MessageRepository
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait MessageRepository: Send + Sync {
    /// Create a new message in the database.
    async fn create(
        &self,
        id: Uuid,
        channel_type: &str,
        channel_id: Option<Uuid>,
        sender_id: Uuid,
        sender_name: &str,
        content: &str,
    ) -> Result<()>;

    /// Get messages from the global channel with cursor-based pagination.
    async fn get_global_messages(&self, limit: i64, before_ts: Option<DateTime<Utc>>) -> Result<Vec<MessageRow>>;

    /// Get messages from a specific channel (group or whisper) with cursor-based pagination.
    async fn get_channel_messages(
        &self,
        channel_type: &str,
        channel_id: Uuid,
        limit: i64,
        before_ts: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageRow>>;

    /// Find the created_at timestamp of a message by ID (used for cursor resolution).
    async fn find_created_at_by_id(&self, id: Uuid) -> Result<Option<DateTime<Utc>>>;
}

// ---------------------------------------------------------------------------
// Outbound port: EventPublisher
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a message sent event to NATS.
    async fn message_sent(
        &self,
        channel_type: &str,
        channel_id: Option<Uuid>,
        sender_id: Uuid,
        content: &str,
    ) -> Result<()>;
}
