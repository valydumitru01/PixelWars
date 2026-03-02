use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::ports::{RoundRepository, VoteAggregate, VoteRepository, VoteRow, VotingWindow};

// ---------------------------------------------------------------------------
// PgVoteRepository
// ---------------------------------------------------------------------------

pub struct PgVoteRepository {
    pool: PgPool,
}

impl PgVoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl VoteRepository for PgVoteRepository {
    async fn find_by_round_voter(&self, round_id: Uuid, voter_id: Uuid) -> Result<Option<VoteRow>> {
        let row = sqlx::query_as!(
            VoteRow,
            "SELECT id, round_id, voter_id, target_id, target_type FROM votes WHERE round_id = $1 AND voter_id = $2",
            round_id,
            voter_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn create(
        &self,
        id: Uuid,
        round_id: Uuid,
        voter_id: Uuid,
        target_id: Uuid,
        target_type: &str,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO votes (id, round_id, voter_id, target_id, target_type) VALUES ($1, $2, $3, $4, $5)",
            id,
            round_id,
            voter_id,
            target_id,
            target_type
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn aggregate_by_round(&self, round_id: Uuid) -> Result<Vec<VoteAggregate>> {
        let rows = sqlx::query!(
            r#"SELECT target_id, target_type, COUNT(*) as "vote_count!"
               FROM votes WHERE round_id = $1
               GROUP BY target_id, target_type
               ORDER BY "vote_count!" DESC"#,
            round_id
        )
        .fetch_all(&self.pool)
        .await?;

        let aggregates = rows
            .into_iter()
            .map(|row| VoteAggregate {
                target_id: row.target_id,
                target_type: row.target_type,
                vote_count: row.vote_count,
            })
            .collect();

        Ok(aggregates)
    }
}

// ---------------------------------------------------------------------------
// PgRoundRepository
// ---------------------------------------------------------------------------

pub struct PgRoundRepository {
    pool: PgPool,
}

impl PgRoundRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl RoundRepository for PgRoundRepository {
    async fn get_voting_window(&self, round_id: Uuid) -> Result<Option<VotingWindow>> {
        let row = sqlx::query!(
            "SELECT voting_starts_at, voting_ends_at FROM rounds WHERE id = $1 AND is_active = true",
            round_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let window = row.map(|r| VotingWindow {
            voting_starts_at: r.voting_starts_at,
            voting_ends_at: r.voting_ends_at,
        });

        Ok(window)
    }

    async fn user_owns_parcel(&self, user_id: Uuid, parcel_id: Uuid) -> Result<bool> {
        let row = sqlx::query!(
            "SELECT id FROM parcels WHERE id = $1 AND user_id = $2",
            parcel_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    async fn user_is_group_member(&self, user_id: Uuid, group_id: Uuid) -> Result<bool> {
        let row = sqlx::query!(
            "SELECT user_id FROM group_members WHERE user_id = $1 AND group_id = $2",
            user_id,
            group_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    async fn target_is_group(&self, target_id: Uuid) -> Result<bool> {
        let row = sqlx::query!(
            "SELECT id FROM groups WHERE id = $1",
            target_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }
}
