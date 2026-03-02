use std::sync::Arc;

use tracing::info;

use crate::ports::{EventPublisher, RoundRepository};

pub struct CloseVoting {
    round_repo: Arc<dyn RoundRepository>,
    events: Arc<dyn EventPublisher>,
}

impl CloseVoting {
    pub fn new(
        round_repo: Arc<dyn RoundRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { round_repo, events }
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        // Find all rounds with expired voting windows
        let expired_rounds = self.round_repo.get_expired_voting_windows().await?;

        for round in &expired_rounds {
            // Close voting window
            self.round_repo.close_voting_window(round.id).await?;

            // Publish event (best-effort)
            if let Err(e) = self.events.voting_closed(round.id).await {
                tracing::warn!(error = %e, round_id = %round.id, "Failed to publish VotingClosed event");
            }

            info!(round_id = %round.id, "Voting window closed");
        }

        Ok(())
    }
}
