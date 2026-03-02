use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ports::{ParcelRepository, ParcelRow};

pub struct PgParcelRepository {
    pool: PgPool,
}

impl PgParcelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl ParcelRepository for PgParcelRepository {
    async fn find_by_id(&self, parcel_id: Uuid) -> Result<Option<ParcelRow>> {
        let row = sqlx::query_as!(
            ParcelRow,
            r#"SELECT id, user_id, round_id, origin_x, origin_y, width, height, description, created_at, updated_at
               FROM parcels WHERE id = $1"#,
            parcel_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_user_round(&self, user_id: Uuid, round_id: Uuid) -> Result<Option<ParcelRow>> {
        let row = sqlx::query_as!(
            ParcelRow,
            r#"SELECT id, user_id, round_id, origin_x, origin_y, width, height, description, created_at, updated_at
               FROM parcels WHERE user_id = $1 AND round_id = $2"#,
            user_id,
            round_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn check_overlap(
        &self,
        round_id: Uuid,
        origin_x: i32,
        origin_y: i32,
        width: i32,
        height: i32,
    ) -> Result<bool> {
        let overlap = sqlx::query!(
            r#"SELECT id FROM parcels
               WHERE round_id = $1
                 AND origin_x < ($2::int4 + $3::int4)
                 AND (origin_x + width) > $2::int4
                 AND origin_y < ($4::int4 + $5::int4)
                 AND (origin_y + height) > $4::int4
               LIMIT 1"#,
            round_id,
            origin_x,
            width,
            origin_y,
            height
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(overlap.is_some())
    }

    async fn create(
        &self,
        id: Uuid,
        user_id: Uuid,
        round_id: Uuid,
        origin_x: i32,
        origin_y: i32,
        width: i32,
        height: i32,
        description: &str,
    ) -> Result<()> {
        let now = Utc::now();

        sqlx::query!(
            r#"INSERT INTO parcels (id, user_id, round_id, origin_x, origin_y, width, height, description, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)"#,
            id,
            user_id,
            round_id,
            origin_x,
            origin_y,
            width,
            height,
            description,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_parcel_timestamp(
        &self,
        parcel_id: Uuid,
        timestamp: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE parcels SET updated_at = $1 WHERE id = $2",
            timestamp,
            parcel_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_user_last_draw(&self, user_id: Uuid, timestamp: DateTime<Utc>) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET last_draw_at = $1 WHERE id = $2",
            timestamp,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
