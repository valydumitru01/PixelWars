use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::chat::SendMessageRequest;
use tracing::info;

use crate::state::ChatState;

pub async fn send_group_message(
    State(state): State<ChatState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Verify user is a member of the group
    // TODO: Publish to group-specific NATS subject
    info!("Group message sent");
    Ok(StatusCode::OK)
}
