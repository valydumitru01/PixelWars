use std::sync::Arc;

use shared_common::errors::AppError;

use crate::domain::commands::CheckOverlapQuery;
use crate::ports::ParcelRepository;

/// Output returned from check overlap use case.
pub struct CheckOverlapOutput {
    pub overlaps: bool,
}

pub struct CheckOverlap {
    parcel_repo: Arc<dyn ParcelRepository>,
}

impl CheckOverlap {
    pub fn new(parcel_repo: Arc<dyn ParcelRepository>) -> Self {
        Self { parcel_repo }
    }

    pub async fn execute(&self, query: CheckOverlapQuery) -> Result<CheckOverlapOutput, AppError> {
        let overlaps = self.parcel_repo
            .check_overlap(
                query.round_id,
                query.origin_x as i32,
                query.origin_y as i32,
                query.width as i32,
                query.height as i32,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(CheckOverlapOutput { overlaps })
    }
}
