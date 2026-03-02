use std::sync::Arc;

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain;
use crate::ports::{EventPublisher, RoundRepository};

pub struct EndRound {
    round_repo: Arc<dyn RoundRepository>,
    events: Arc<dyn EventPublisher>,
}

impl EndRound {
    pub fn new(
        round_repo: Arc<dyn RoundRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { round_repo, events }
    }

    pub async fn execute(&self, round_id: Uuid) -> anyhow::Result<()> {
        let now = Utc::now();
        let voting_ends = now + chrono::Duration::days(domain::VOTING_WINDOW_DAYS);

        // Deactivate round and open voting window
        self.round_repo.deactivate(round_id).await?;
        self.round_repo.open_voting_window(round_id, now, voting_ends).await?;

        // Publish events (best-effort)
        if let Err(e) = self.events.round_ended(round_id).await {
            tracing::warn!(error = %e, round_id = %round_id, "Failed to publish RoundEnded event");
        }

        if let Err(e) = self.events.voting_opened(round_id).await {
            tracing::warn!(error = %e, round_id = %round_id, "Failed to publish VotingOpened event");
        }

        info!(round_id = %round_id, voting_until = %voting_ends, "Round ended, voting opened");
        Ok(())
    }
}
