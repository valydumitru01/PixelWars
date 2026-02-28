use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::vote::CastVoteRequest;
use tracing::info;

use crate::state::VotingState;

pub async fn cast_vote(
    State(state): State<VotingState>,
    Json(req): Json<CastVoteRequest>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Verify voting window is open
    // TODO: Verify user hasn't already voted this round
    // TODO: Insert vote into PostgreSQL
    // TODO: Publish VoteCast event
    info!(target_id = %req.target_id, "Vote cast");
    Ok(StatusCode::CREATED)
}
