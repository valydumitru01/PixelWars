use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_GROUP_SIZE: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub round_id: Uuid,
    pub member_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInvite {
    pub id: Uuid,
    pub group_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub status: InviteStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InviteStatus {
    Pending,
    Accepted,
    Declined,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteRequest {
    pub to_user_id: Uuid,
}
