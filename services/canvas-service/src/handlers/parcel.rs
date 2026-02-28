use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::parcel::ClaimParcelRequest;
use tracing::info;

use crate::{domain::parcel_claim, state::CanvasState};

pub async fn claim_parcel(
    State(state): State<CanvasState>,
    Json(req): Json<ClaimParcelRequest>,
) -> Result<StatusCode, StatusCode> {
    parcel_claim::validate_claim(&req).map_err(|_| StatusCode::BAD_REQUEST)?;

    // TODO: Check for overlapping claims in DB
    // TODO: Insert parcel into PostgreSQL
    // TODO: Publish ParcelClaimed event

    info!(
        origin = %format!("({},{})", req.origin_x, req.origin_y),
        size = %format!("{}x{}", req.width, req.height),
        "Parcel claimed"
    );

    Ok(StatusCode::CREATED)
}
