use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ports::{MessageRepository, MessageRow};

pub struct PgMessageRepository {
    pool: PgPool,
}

impl PgMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl MessageRepository for PgMessageRepository {
    async fn create(
        &self,
        id: Uuid,
        channel_type: &str,
        channel_id: Option<Uuid>,
        sender_id: Uuid,
        sender_name: &str,
        content: &str,
    ) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO chat_messages (id, channel_type, channel_id, sender_id, sender_name, content)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
            id,
            channel_type,
            channel_id,
            sender_id,
            sender_name,
            content
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_global_messages(
        &self,
        limit: i64,
        before_ts: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageRow>> {
        let rows = sqlx::query!(
            r#"SELECT id, sender_id, sender_name, channel_type, channel_id, content, created_at
               FROM chat_messages
               WHERE channel_type = 'global'
                 AND ($2::timestamptz IS NULL OR created_at < $2)
               ORDER BY created_at DESC
               LIMIT $1"#,
            limit,
            before_ts,
        )
        .fetch_all(&self.pool)
        .await?;

        let messages = rows
            .into_iter()
            .map(|r| MessageRow {
                id: r.id,
                sender_id: r.sender_id,
                sender_name: r.sender_name,
                channel_type: r.channel_type,
                channel_id: r.channel_id,
                content: r.content,
                created_at: r.created_at,
            })
            .collect();

        Ok(messages)
    }

    async fn get_channel_messages(
        &self,
        channel_type: &str,
        channel_id: Uuid,
        limit: i64,
        before_ts: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageRow>> {
        let rows = sqlx::query!(
            r#"SELECT id, sender_id, sender_name, channel_type, channel_id, content, created_at
               FROM chat_messages
               WHERE channel_type = $1 AND channel_id = $2
                 AND ($4::timestamptz IS NULL OR created_at < $4)
               ORDER BY created_at DESC
               LIMIT $3"#,
            channel_type,
            channel_id,
            limit,
            before_ts,
        )
        .fetch_all(&self.pool)
        .await?;

        let messages = rows
            .into_iter()
            .map(|r| MessageRow {
                id: r.id,
                sender_id: r.sender_id,
                sender_name: r.sender_name,
                channel_type: r.channel_type,
                channel_id: r.channel_id,
                content: r.content,
                created_at: r.created_at,
            })
            .collect();

        Ok(messages)
    }

    async fn find_created_at_by_id(&self, id: Uuid) -> Result<Option<DateTime<Utc>>> {
        let row = sqlx::query!("SELECT created_at FROM chat_messages WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.created_at))
    }
}
