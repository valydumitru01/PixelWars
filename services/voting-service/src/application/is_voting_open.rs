use std::sync::Arc;

use shared_common::errors::AppError;
use uuid::Uuid;

use crate::ports::RoundRepository;

pub struct IsVotingOpen {
    round_repo: Arc<dyn RoundRepository>,
}

impl IsVotingOpen {
    pub fn new(round_repo: Arc<dyn RoundRepository>) -> Self {
        Self { round_repo }
    }

    pub async fn execute(&self, round_id: Uuid) -> Result<bool, AppError> {
        let voting_window = self
            .round_repo
            .get_voting_window(round_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let is_open = match voting_window {
            Some(window) => {
                let now = chrono::Utc::now();
                matches!(
                    (window.voting_starts_at, window.voting_ends_at),
                    (Some(start), Some(end)) if now >= start && now <= end
                )
            }
            None => false,
        };

        Ok(is_open)
    }
}
