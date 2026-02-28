use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::pixel::BatchPixelUpdate;
use tracing::info;

use crate::state::CanvasState;

pub async fn update_pixels(
    State(state): State<CanvasState>,
    Json(req): Json<BatchPixelUpdate>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Verify user owns the parcel
    // TODO: Apply pixel updates to canvas buffer & Redis
    // TODO: Publish PixelUpdated events for real-time broadcast
    // TODO: Update last_draw_at timestamp

    info!(
        parcel_id = %req.parcel_id,
        pixel_count = req.pixels.len(),
        "Pixels updated"
    );

    Ok(StatusCode::OK)
}
