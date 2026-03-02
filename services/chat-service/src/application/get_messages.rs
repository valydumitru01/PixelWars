use std::sync::Arc;

use shared_common::errors::AppError;
use uuid::Uuid;

use crate::domain::commands::GetMessagesQuery;
use crate::ports::{MessageRepository, MessageRow};

// ---------------------------------------------------------------------------
// Pagination limits
// ---------------------------------------------------------------------------

/// Hard upper bound — prevents a single RPC from returning the entire history.
const MESSAGES_MAX_PAGE_SIZE: u32 = 100;

/// Hard lower bound — a limit of 0 would be a no-op and is almost certainly a bug.
const MESSAGES_MIN_PAGE_SIZE: u32 = 1;

/// Default page size if none is specified.
const MESSAGES_DEFAULT_PAGE_SIZE: u32 = 50;

/// Output returned after retrieving messages.
pub struct GetMessagesOutput {
    pub messages: Vec<MessageRow>,
}

pub struct GetMessages {
    message_repo: Arc<dyn MessageRepository>,
}

impl GetMessages {
    pub fn new(message_repo: Arc<dyn MessageRepository>) -> Self {
        Self { message_repo }
    }

    pub async fn execute(&self, query: GetMessagesQuery) -> Result<GetMessagesOutput, AppError> {
        // 1. Resolve the cursor (before_id) to a timestamp
        let cursor_ts = if let Some(before_id) = query.before_id {
            self.message_repo
                .find_created_at_by_id(before_id)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            None
        };

        // 2. Clamp limit to allowed range
        let limit = query.limit
            .clamp(MESSAGES_MIN_PAGE_SIZE, MESSAGES_MAX_PAGE_SIZE) as i64;

        // 3. Query messages
        let messages = if query.channel_type == "global" {
            self.message_repo
                .get_global_messages(limit, cursor_ts)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            let channel_id = query.channel_id
                .ok_or(AppError::Validation("channel_id required for non-global channels".into()))?;

            self.message_repo
                .get_channel_messages(&query.channel_type, channel_id, limit, cursor_ts)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
        };

        Ok(GetMessagesOutput { messages })
    }
}
