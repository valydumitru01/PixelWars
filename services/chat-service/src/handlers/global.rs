use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::chat::{ChatChannel, SendMessageRequest};
use tracing::info;

use crate::state::ChatState;

pub async fn send_global_message(
    State(state): State<ChatState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Store message in Redis for recent history
    // TODO: Publish via NATS for real-time delivery to WebSocket connections
    info!(content_len = req.content.len(), "Global message sent");
    Ok(StatusCode::OK)
}
