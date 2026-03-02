use std::sync::Arc;

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain;
use crate::ports::{EventPublisher, RoundRepository};

pub struct StartRound {
    round_repo: Arc<dyn RoundRepository>,
    events: Arc<dyn EventPublisher>,
}

impl StartRound {
    pub fn new(
        round_repo: Arc<dyn RoundRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { round_repo, events }
    }

    pub async fn execute(&self) -> anyhow::Result<Uuid> {
        // Reset disqualifications for all users
        self.round_repo.reset_disqualifications().await?;

        // Create new round
        let round_id = Uuid::new_v4();
        let now = Utc::now();
        let ends_at = now + chrono::Duration::days(domain::ROUND_DURATION_DAYS);

        self.round_repo.create_new(round_id, ends_at).await?;

        // Publish event (best-effort)
        if let Err(e) = self.events.round_started(round_id).await {
            tracing::warn!(error = %e, round_id = %round_id, "Failed to publish RoundStarted event");
        }

        info!(round_id = %round_id, ends_at = %ends_at, "New round started");
        Ok(round_id)
    }
}
