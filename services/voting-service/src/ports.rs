use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain data returned by repositories
// ---------------------------------------------------------------------------

/// Represents a vote row from the database.
pub struct VoteRow {
    pub id: Uuid,
    pub round_id: Uuid,
    pub voter_id: Uuid,
    pub target_id: Uuid,
    pub target_type: String, // "parcel" or "group"
}

/// Aggregated vote counts for a target in a round.
pub struct VoteAggregate {
    pub target_id: Uuid,
    pub target_type: String,
    pub vote_count: i64,
}

/// Voting window times for a round.
pub struct VotingWindow {
    pub voting_starts_at: Option<DateTime<Utc>>,
    pub voting_ends_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Outbound port: VoteRepository
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait VoteRepository: Send + Sync {
    /// Find an existing vote by round and voter.
    async fn find_by_round_voter(&self, round_id: Uuid, voter_id: Uuid) -> Result<Option<VoteRow>>;

    /// Create a new vote record.
    async fn create(
        &self,
        id: Uuid,
        round_id: Uuid,
        voter_id: Uuid,
        target_id: Uuid,
        target_type: &str,
    ) -> Result<()>;

    /// Aggregate votes by round, grouped by target_id.
    async fn aggregate_by_round(&self, round_id: Uuid) -> Result<Vec<VoteAggregate>>;
}

// ---------------------------------------------------------------------------
// Outbound port: RoundRepository
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait RoundRepository: Send + Sync {
    /// Get the voting window (start/end times) for a round.
    async fn get_voting_window(&self, round_id: Uuid) -> Result<Option<VotingWindow>>;

    /// Check if a user owns a parcel.
    async fn user_owns_parcel(&self, user_id: Uuid, parcel_id: Uuid) -> Result<bool>;

    /// Check if a user is a member of a group.
    async fn user_is_group_member(&self, user_id: Uuid, group_id: Uuid) -> Result<bool>;

    /// Check if a target ID is a group (returns true if it is).
    async fn target_is_group(&self, target_id: Uuid) -> Result<bool>;
}

// ---------------------------------------------------------------------------
// Outbound port: EventPublisher
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a VoteCast event.
    async fn vote_cast(&self, voter_id: Uuid, target_id: Uuid) -> Result<()>;
}
