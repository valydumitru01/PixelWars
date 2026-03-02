use std::sync::Arc;

use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::domain;
use crate::ports::{EventPublisher, ParcelRepository, UserRepository};

pub struct RunActivityCheck {
    user_repo: Arc<dyn UserRepository>,
    parcel_repo: Arc<dyn ParcelRepository>,
    events: Arc<dyn EventPublisher>,
}

impl RunActivityCheck {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        parcel_repo: Arc<dyn ParcelRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            user_repo,
            parcel_repo,
            events,
        }
    }

    pub async fn execute(&self, round_id: Uuid) -> anyhow::Result<()> {
        let cutoff = Utc::now() - chrono::Duration::days(domain::INACTIVITY_THRESHOLD_DAYS);

        // Find all inactive users in the round
        let inactive_users = self.user_repo
            .find_inactive_since(cutoff, round_id)
            .await?;

        let count = inactive_users.len();

        // Process each inactive user
        for user in &inactive_users {
            // Mark disqualified
            self.user_repo.disqualify(user.id).await?;

            // Release their parcel
            self.parcel_repo.delete_by_user_round(user.id, round_id).await?;

            // Publish event (best-effort)
            if let Err(e) = self.events
                .user_disqualified(user.id, round_id, "No drawing activity for 3 days")
                .await
            {
                warn!(error = %e, user_id = %user.id, "Failed to publish disqualification event");
            }
        }

        info!(round_id = %round_id, disqualified = count, "Activity check completed");
        Ok(())
    }
}
