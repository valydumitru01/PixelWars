use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::ports::{GroupRepository, GroupRow, InviteRepository, InviteRow};

// ---------------------------------------------------------------------------
// PgGroupRepository
// ---------------------------------------------------------------------------

pub struct PgGroupRepository {
    pool: PgPool,
}

impl PgGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl GroupRepository for PgGroupRepository {
    async fn create(&self, id: Uuid, name: &str, creator_id: Uuid, round_id: Uuid) -> Result<()> {
        let now = chrono::Utc::now();
        sqlx::query!(
            "INSERT INTO groups (id, name, creator_id, round_id, created_at) VALUES ($1, $2, $3, $4, $5)",
            id,
            name,
            creator_id,
            round_id,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<GroupRow>> {
        let row = sqlx::query!(
            "SELECT id, name, creator_id, round_id, created_at FROM groups WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| GroupRow {
            id: r.id,
            name: r.name,
            creator_id: r.creator_id,
            round_id: r.round_id,
            created_at: r.created_at,
        }))
    }

    async fn find_user_group(&self, user_id: Uuid, round_id: Uuid) -> Result<Option<GroupRow>> {
        let row = sqlx::query!(
            r#"SELECT g.id, g.name, g.creator_id, g.round_id, g.created_at
               FROM groups g
               JOIN group_members gm ON gm.group_id = g.id
               WHERE gm.user_id = $1 AND g.round_id = $2"#,
            user_id,
            round_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| GroupRow {
            id: r.id,
            name: r.name,
            creator_id: r.creator_id,
            round_id: r.round_id,
            created_at: r.created_at,
        }))
    }

    async fn get_member_count(&self, group_id: Uuid) -> Result<i64> {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) as "count!" FROM group_members WHERE group_id = $1"#,
            group_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count)
    }

    async fn get_member_ids(&self, group_id: Uuid) -> Result<Vec<Uuid>> {
        let rows = sqlx::query!(
            "SELECT user_id FROM group_members WHERE group_id = $1",
            group_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.user_id).collect())
    }

    async fn add_member(&self, group_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            "INSERT INTO group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            group_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn user_has_parcel(&self, user_id: Uuid, round_id: Uuid) -> Result<bool> {
        let row = sqlx::query!(
            "SELECT id FROM parcels WHERE user_id = $1 AND round_id = $2",
            user_id,
            round_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }
}

// ---------------------------------------------------------------------------
// PgInviteRepository
// ---------------------------------------------------------------------------

pub struct PgInviteRepository {
    pool: PgPool,
}

impl PgInviteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl InviteRepository for PgInviteRepository {
    async fn create(
        &self,
        id: Uuid,
        group_id: Uuid,
        from_user: Uuid,
        to_user: Uuid,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO group_invites (id, group_id, from_user, to_user, status) VALUES ($1, $2, $3, $4, 'pending')",
            id,
            group_id,
            from_user,
            to_user
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<InviteRow>> {
        let row = sqlx::query!(
            "SELECT id, group_id, from_user, to_user, status FROM group_invites WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| InviteRow {
            id: r.id,
            group_id: r.group_id,
            from_user: r.from_user,
            to_user: r.to_user,
            status: r.status,
        }))
    }

    async fn accept(&self, id: Uuid) -> Result<()> {
        sqlx::query!("UPDATE group_invites SET status = 'accepted' WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
