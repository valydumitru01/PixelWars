use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::vote::VoteResults;

use crate::state::VotingState;

pub async fn get_results(
    State(state): State<VotingState>,
) -> Result<Json<VoteResults>, StatusCode> {
    // TODO: Fetch current round results from DB
    // TODO: Tally votes using tallying::tally_votes
    Err(StatusCode::NOT_IMPLEMENTED)
}
