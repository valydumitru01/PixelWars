use axum::{extract::State, http::StatusCode};

use crate::state::GroupState;

pub async fn leave_group(
    State(state): State<GroupState>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Remove user from group
    // TODO: If last member, disband group
    Ok(StatusCode::OK)
}
