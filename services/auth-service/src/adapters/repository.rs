use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::ports::{UserRepository, UserRow};

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl UserRepository for PgUserRepository {
    async fn exists_by_email_or_username(&self, email: &str, username: &str) -> Result<bool> {
        let row = sqlx::query!(
            "SELECT id FROM users WHERE email = $1 OR username = $2",
            email,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    async fn create(&self, id: Uuid, username: &str, email: &str, password_hash: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)",
            id,
            username,
            email,
            password_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<UserRow>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"SELECT id, username, email, password_hash, is_active, is_disqualified, last_draw_at
               FROM users WHERE email = $1 AND is_active = true"#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRow>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"SELECT id, username, email, password_hash, is_active, is_disqualified, last_draw_at
               FROM users WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }
}
