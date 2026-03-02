use anyhow::Result;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain data returned by repositories
// ---------------------------------------------------------------------------

pub struct GroupRow {
    pub id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub round_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct InviteRow {
    pub id: Uuid,
    pub group_id: Uuid,
    pub from_user: Uuid,
    pub to_user: Uuid,
    pub status: String,
}

// ---------------------------------------------------------------------------
// Outbound port: GroupRepository
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait GroupRepository: Send + Sync {
    /// Create a new group and return its ID.
    async fn create(&self, id: Uuid, name: &str, creator_id: Uuid, round_id: Uuid) -> Result<()>;

    /// Find a group by ID.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GroupRow>>;

    /// Find a user's group in a given round.
    async fn find_user_group(&self, user_id: Uuid, round_id: Uuid) -> Result<Option<GroupRow>>;

    /// Get the count of members in a group.
    async fn get_member_count(&self, group_id: Uuid) -> Result<i64>;

    /// Get all member IDs in a group.
    async fn get_member_ids(&self, group_id: Uuid) -> Result<Vec<Uuid>>;

    /// Add a member to a group.
    async fn add_member(&self, group_id: Uuid, user_id: Uuid) -> Result<()>;

    /// Check if a user has a parcel in a given round.
    async fn user_has_parcel(&self, user_id: Uuid, round_id: Uuid) -> Result<bool>;
}

// ---------------------------------------------------------------------------
// Outbound port: InviteRepository
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait InviteRepository: Send + Sync {
    /// Create a new invite.
    async fn create(&self, id: Uuid, group_id: Uuid, from_user: Uuid, to_user: Uuid) -> Result<()>;

    /// Find an invite by ID.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<InviteRow>>;

    /// Update invite status to accepted.
    async fn accept(&self, id: Uuid) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Outbound port: EventPublisher
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn group_created(&self, group_id: Uuid, creator_id: Uuid) -> Result<()>;

    async fn invite_sent(
        &self,
        group_id: Uuid,
        from_user: Uuid,
        to_user: Uuid,
    ) -> Result<()>;

    async fn invite_accepted(&self, group_id: Uuid, user_id: Uuid) -> Result<()>;
}
