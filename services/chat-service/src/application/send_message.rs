use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::commands::SendMessageCommand;
use crate::domain::validation;
use crate::ports::{EventPublisher, MessageRepository};

/// Output returned after a successful message send.
pub struct SendMessageOutput {
    pub message_id: Uuid,
}

pub struct SendMessage {
    message_repo: Arc<dyn MessageRepository>,
    events: Arc<dyn EventPublisher>,
}

impl SendMessage {
    pub fn new(
        message_repo: Arc<dyn MessageRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { message_repo, events }
    }

    pub async fn execute(&self, cmd: SendMessageCommand) -> Result<SendMessageOutput, AppError> {
        // 1. Validate content
        validation::validate_content(&cmd.content)?;

        // 2. Generate message ID
        let message_id = Uuid::new_v4();

        // 3. Create message in database
        self.message_repo
            .create(
                message_id,
                &cmd.channel_type,
                cmd.channel_id,
                cmd.sender_id,
                &cmd.sender_name,
                &cmd.content,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 4. Publish event (best-effort — don't fail if NATS is down)
        if let Err(e) = self.events
            .message_sent(&cmd.channel_type, cmd.channel_id, cmd.sender_id, &cmd.content)
            .await
        {
            tracing::warn!(error = %e, "Failed to publish ChatMessage event");
        }

        info!(
            message_id = %message_id,
            channel_type = %cmd.channel_type,
            sender_id = %cmd.sender_id,
            "Chat message created"
        );

        Ok(SendMessageOutput { message_id })
    }
}
