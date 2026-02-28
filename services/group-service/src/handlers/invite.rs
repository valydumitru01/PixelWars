use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::group::{InviteRequest, MAX_GROUP_SIZE};
use tracing::info;

use crate::state::GroupState;

pub async fn invite_member(
    State(state): State<GroupState>,
    Json(req): Json<InviteRequest>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Verify group exists and caller is a member
    // TODO: Verify group size < MAX_GROUP_SIZE (10)
    // TODO: Verify target user has an adjacent parcel
    // TODO: Insert invite into PostgreSQL
    // TODO: Publish GroupInviteSent event
    info!(to_user = %req.to_user_id, "Group invite sent");
    Ok(StatusCode::CREATED)
}

pub async fn accept_invite(
    State(state): State<GroupState>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Accept the invite, add user to group
    // TODO: Publish GroupInviteAccepted event
    Ok(StatusCode::OK)
}
