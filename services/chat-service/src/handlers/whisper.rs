use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::chat::SendMessageRequest;
use tracing::info;

use crate::state::ChatState;

pub async fn send_whisper(
    State(state): State<ChatState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Deliver directly to target user via NATS
    info!("Whisper sent");
    Ok(StatusCode::OK)
}
