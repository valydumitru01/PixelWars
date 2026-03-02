use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain data returned by repositories
// ---------------------------------------------------------------------------

pub struct InactiveUserRow {
    pub id: Uuid,
    pub round_id: Uuid,
}

pub struct RoundRow {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub voting_starts_at: Option<DateTime<Utc>>,
    pub voting_ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

// ---------------------------------------------------------------------------
// Outbound port: UserRepository
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    /// Find all inactive users (no drawing activity for 3 days) in the current round.
    async fn find_inactive_since(
        &self,
        threshold: DateTime<Utc>,
        round_id: Uuid,
    ) -> Result<Vec<InactiveUserRow>>;

    /// Mark a user as disqualified.
    async fn disqualify(&self, user_id: Uuid) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Outbound port: ParcelRepository
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait ParcelRepository: Send + Sync {
    /// Delete all parcels for a user in a specific round.
    async fn delete_by_user_round(&self, user_id: Uuid, round_id: Uuid) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Outbound port: RoundRepository
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait RoundRepository: Send + Sync {
    /// Get the currently active round, if any.
    async fn get_active(&self) -> Result<Option<RoundRow>>;

    /// Deactivate a round (end it).
    async fn deactivate(&self, round_id: Uuid) -> Result<()>;

    /// Create a new round.
    async fn create_new(&self, id: Uuid, ends_at: DateTime<Utc>) -> Result<()>;

    /// Reset disqualifications for all active users and clear last_draw_at.
    async fn reset_disqualifications(&self) -> Result<()>;

    /// Open the voting window for a round.
    async fn open_voting_window(
        &self,
        round_id: Uuid,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<()>;

    /// Close the voting window for a round.
    async fn close_voting_window(&self, round_id: Uuid) -> Result<()>;

    /// Get all rounds with expired voting windows that are not active.
    async fn get_expired_voting_windows(&self) -> Result<Vec<RoundRow>>;
}

// ---------------------------------------------------------------------------
// Outbound port: EventPublisher
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn user_disqualified(&self, user_id: Uuid, round_id: Uuid, reason: &str) -> Result<()>;
    async fn round_started(&self, round_id: Uuid) -> Result<()>;
    async fn round_ended(&self, round_id: Uuid) -> Result<()>;
    async fn voting_opened(&self, round_id: Uuid) -> Result<()>;
    async fn voting_closed(&self, round_id: Uuid) -> Result<()>;
}
