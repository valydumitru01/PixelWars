use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub id: Uuid,
    pub round_id: Uuid,
    pub voter_id: Uuid,
    /// The parcel or group being voted for.
    pub target_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CastVoteRequest {
    pub target_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct VoteResults {
    pub round_id: Uuid,
    pub rankings: Vec<VoteRanking>,
}

#[derive(Debug, Serialize)]
pub struct VoteRanking {
    pub target_id: Uuid,
    pub vote_count: u64,
    pub rank: u32,
}
