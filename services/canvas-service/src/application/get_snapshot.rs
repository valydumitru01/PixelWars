use std::sync::Arc;

use shared_common::errors::AppError;

use crate::domain::commands::SnapshotQuery;
use crate::ports::PixelCache;

/// Hard upper bound on snapshot dimensions — prevents accidental or malicious
/// requests that would stream hundreds of MB of pixel data in one RPC call.
/// The gateway fills in API-level defaults; this is the safety cap.
pub const SNAPSHOT_MAX_SIZE: u32 = 512;

/// Output returned from get snapshot use case.
pub struct GetSnapshotOutput {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct GetSnapshot {
    pixel_cache: Arc<dyn PixelCache>,
}

impl GetSnapshot {
    pub fn new(pixel_cache: Arc<dyn PixelCache>) -> Self {
        Self { pixel_cache }
    }

    pub async fn execute(&self, query: SnapshotQuery) -> Result<GetSnapshotOutput, AppError> {
        let w = query.width.min(SNAPSHOT_MAX_SIZE);
        let h = query.height.min(SNAPSHOT_MAX_SIZE);

        let data = self.pixel_cache
            .get_snapshot_region(query.x, query.y, w, h)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(GetSnapshotOutput { data, width: w, height: h })
    }
}
