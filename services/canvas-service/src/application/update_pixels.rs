use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::commands::UpdatePixelsCommand;
use crate::ports::{EventPublisher, ParcelRepository, PixelCache};

/// Output returned from update pixels use case.
pub struct UpdatePixelsOutput {
    pub updated_count: u32,
}

pub struct UpdatePixels {
    parcel_repo: Arc<dyn ParcelRepository>,
    pixel_cache: Arc<dyn PixelCache>,
    events: Arc<dyn EventPublisher>,
}

impl UpdatePixels {
    pub fn new(
        parcel_repo: Arc<dyn ParcelRepository>,
        pixel_cache: Arc<dyn PixelCache>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { parcel_repo, pixel_cache, events }
    }

    pub async fn execute(&self, cmd: UpdatePixelsCommand) -> Result<UpdatePixelsOutput, AppError> {
        // 1. Verify user owns the parcel
        let parcel = self.parcel_repo
            .find_by_id(cmd.parcel_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or(AppError::NotFound("Parcel not found".into()))?;

        if parcel.user_id != cmd.user_id {
            return Err(AppError::Forbidden("You do not own this parcel".into()));
        }

        // 2. Set pixels in cache
        let mut count: u32 = 0;
        for px in &cmd.pixels {
            // Convert local coords to global canvas coords
            let global_x = parcel.origin_x as u32 + px.local_x;
            let global_y = parcel.origin_y as u32 + px.local_y;

            // Bounds check within parcel
            if px.local_x >= parcel.width as u32 || px.local_y >= parcel.height as u32 {
                continue;
            }

            self.pixel_cache
                .set_pixel(global_x, global_y, px.color)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            count += 1;

            // Publish event for each pixel (best-effort)
            if let Err(e) = self.events
                .pixels_updated(cmd.parcel_id, global_x, global_y, px.color)
                .await
            {
                tracing::warn!(error = %e, "Failed to publish PixelsUpdated event");
            }
        }

        // 3. Update timestamps
        let now = chrono::Utc::now();

        self.parcel_repo
            .update_parcel_timestamp(cmd.parcel_id, now)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        self.parcel_repo
            .update_user_last_draw(cmd.user_id, now)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        info!(parcel_id = %cmd.parcel_id, updated = count, "Pixels updated");

        Ok(UpdatePixelsOutput { updated_count: count })
    }
}
