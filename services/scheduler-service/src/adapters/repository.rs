use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ports::{
    InactiveUserRow, ParcelRepository, RoundRepository, RoundRow, UserRepository,
};

// ---------------------------------------------------------------------------
// PgUserRepository
// ---------------------------------------------------------------------------

pub struct PgUserRepository {
    db: PgPool,
}

impl PgUserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    async fn find_inactive_since(
        &self,
        threshold: DateTime<Utc>,
        round_id: Uuid,
    ) -> Result<Vec<InactiveUserRow>> {
        let rows = sqlx::query!(
            r#"SELECT u.id, p.round_id
               FROM users u
               JOIN parcels p ON p.user_id = u.id AND p.round_id = $1
               WHERE u.is_active = true
                 AND u.is_disqualified = false
                 AND (u.last_draw_at IS NULL OR u.last_draw_at < $2)"#,
            round_id,
            threshold
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| InactiveUserRow {
                id: row.id,
                round_id: row.round_id,
            })
            .collect())
    }

    async fn disqualify(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!("UPDATE users SET is_disqualified = true WHERE id = $1", user_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PgParcelRepository
// ---------------------------------------------------------------------------

pub struct PgParcelRepository {
    db: PgPool,
}

impl PgParcelRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl ParcelRepository for PgParcelRepository {
    async fn delete_by_user_round(&self, user_id: Uuid, round_id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM parcels WHERE user_id = $1 AND round_id = $2",
            user_id,
            round_id
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PgRoundRepository
// ---------------------------------------------------------------------------

pub struct PgRoundRepository {
    db: PgPool,
}

impl PgRoundRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl RoundRepository for PgRoundRepository {
    async fn get_active(&self) -> Result<Option<RoundRow>> {
        let row = sqlx::query!(
            "SELECT id, started_at, ends_at, voting_starts_at, voting_ends_at, is_active FROM rounds WHERE is_active = true LIMIT 1"
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| RoundRow {
            id: r.id,
            started_at: r.started_at,
            ends_at: r.ends_at,
            voting_starts_at: r.voting_starts_at,
            voting_ends_at: r.voting_ends_at,
            is_active: r.is_active,
        }))
    }

    async fn deactivate(&self, round_id: Uuid) -> Result<()> {
        sqlx::query!("UPDATE rounds SET is_active = false WHERE id = $1", round_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn create_new(&self, id: Uuid, ends_at: DateTime<Utc>) -> Result<()> {
        let now = Utc::now();
        sqlx::query!(
            "INSERT INTO rounds (id, started_at, ends_at, is_active) VALUES ($1, $2, $3, true)",
            id,
            now,
            ends_at
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn reset_disqualifications(&self) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET is_disqualified = false, last_draw_at = NULL WHERE is_active = true"
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn open_voting_window(
        &self,
        round_id: Uuid,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE rounds SET voting_starts_at = $1, voting_ends_at = $2 WHERE id = $3",
            starts_at,
            ends_at,
            round_id
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn close_voting_window(&self, round_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE rounds SET voting_starts_at = NULL, voting_ends_at = NULL WHERE id = $1",
            round_id
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_expired_voting_windows(&self) -> Result<Vec<RoundRow>> {
        let now = Utc::now();
        let rows = sqlx::query!(
            "SELECT id, started_at, ends_at, voting_starts_at, voting_ends_at, is_active FROM rounds WHERE voting_ends_at IS NOT NULL AND voting_ends_at < $1 AND is_active = false",
            now
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| RoundRow {
                id: r.id,
                started_at: r.started_at,
                ends_at: r.ends_at,
                voting_starts_at: r.voting_starts_at,
                voting_ends_at: r.voting_ends_at,
                is_active: r.is_active,
            })
            .collect())
    }
}
