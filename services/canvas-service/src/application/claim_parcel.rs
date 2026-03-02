use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::commands::ClaimParcelCommand;
use crate::domain::contiguity::is_valid_rectangle;
use crate::ports::{EventPublisher, ParcelRepository};

/// Output returned after a successful parcel claim.
pub struct ClaimParcelOutput {
    pub parcel_id: Uuid,
    pub user_id: Uuid,
    pub round_id: Uuid,
    pub origin_x: u32,
    pub origin_y: u32,
    pub width: u32,
    pub height: u32,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct ClaimParcel {
    parcel_repo: Arc<dyn ParcelRepository>,
    events: Arc<dyn EventPublisher>,
}

impl ClaimParcel {
    pub fn new(
        parcel_repo: Arc<dyn ParcelRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { parcel_repo, events }
    }

    pub async fn execute(&self, cmd: ClaimParcelCommand) -> Result<ClaimParcelOutput, AppError> {
        // 1. Validate dimensions
        if !is_valid_rectangle(cmd.origin_x, cmd.origin_y, cmd.width, cmd.height) {
            return Err(AppError::Validation(
                "Parcel must be exactly 10,000 pixels and fit within 10k×10k canvas".to_string(),
            ));
        }

        // 2. Validate description
        if cmd.description.is_empty() || cmd.description.len() > 500 {
            return Err(AppError::Validation(
                "Description must be 1-500 characters".to_string(),
            ));
        }

        // 3. Check user hasn't already claimed a parcel this round
        let existing = self.parcel_repo
            .find_by_user_round(cmd.user_id, cmd.round_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if existing.is_some() {
            return Err(AppError::Conflict("You already claimed a parcel this round".into()));
        }

        // 4. Check overlap with existing parcels
        let overlaps = self.parcel_repo
            .check_overlap(
                cmd.round_id,
                cmd.origin_x as i32,
                cmd.origin_y as i32,
                cmd.width as i32,
                cmd.height as i32,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if overlaps {
            return Err(AppError::Conflict("Region overlaps with an existing parcel".into()));
        }

        // 5. Create parcel
        let parcel_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        self.parcel_repo
            .create(
                parcel_id,
                cmd.user_id,
                cmd.round_id,
                cmd.origin_x as i32,
                cmd.origin_y as i32,
                cmd.width as i32,
                cmd.height as i32,
                &cmd.description,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 6. Publish event (best-effort — don't fail the operation if NATS is down)
        if let Err(e) = self.events
            .parcel_claimed(cmd.user_id, parcel_id, cmd.origin_x, cmd.origin_y, cmd.width, cmd.height)
            .await
        {
            tracing::warn!(error = %e, "Failed to publish ParcelClaimed event");
        }

        info!(parcel_id = %parcel_id, user_id = %cmd.user_id, "Parcel claimed");

        Ok(ClaimParcelOutput {
            parcel_id,
            user_id: cmd.user_id,
            round_id: cmd.round_id,
            origin_x: cmd.origin_x,
            origin_y: cmd.origin_y,
            width: cmd.width,
            height: cmd.height,
            description: cmd.description,
            created_at: now,
        })
    }
}
