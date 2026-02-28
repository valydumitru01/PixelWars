use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::group::CreateGroupRequest;
use tracing::info;
use uuid::Uuid;

use crate::state::GroupState;

pub async fn create_group(
    State(state): State<GroupState>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<StatusCode, StatusCode> {
    let group_id = Uuid::new_v4();
    // TODO: Insert group into PostgreSQL with creator as first member
    // TODO: Publish GroupCreated event
    info!(group_id = %group_id, name = %req.name, "Group created");
    Ok(StatusCode::CREATED)
}
