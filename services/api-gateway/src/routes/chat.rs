use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Extension, Json, Router,
};
use shared_common::{errors::AppError, models::user::UserClaims};
use crate::clients::chat::*;
use crate::state::AppState;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// API-level default for the messages page size.
// The chat-service enforces its own hard caps independently.
// ---------------------------------------------------------------------------
const MESSAGES_DEFAULT_PAGE_SIZE: u32 = 50;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/messages", post(send_message))
        .route("/messages/{channel_type}", get(get_messages))
}

#[derive(serde::Deserialize)]
pub struct SendBody {
    pub channel_type: String,
    pub channel_id: Option<Uuid>,  // group_id or recipient user_id; absent for global
    pub content: String,
}

async fn send_message(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(body): Json<SendBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.chat_client.clone();
    let reply = client
        .send_message(SendMessageRequest {
            sender_id: claims.sub.to_string(),                                    // Uuid → String at gRPC boundary
            sender_name: claims.username.clone(),
            channel_type: body.channel_type,
            channel_id: body.channel_id.map(|id| id.to_string()).unwrap_or_default(), // Uuid → String at gRPC boundary
            content: body.content,
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({ "message_id": reply.message_id })))
}

#[derive(serde::Deserialize)]
pub struct MessagesQuery {
    pub channel_id: Option<Uuid>,  // group_id or recipient user_id; absent for global
    pub limit: Option<u32>,
    pub before_id: Option<Uuid>,   // cursor for pagination: fetch messages before this message ID
}

async fn get_messages(
    State(state): State<AppState>,
    Path(channel_type): Path<String>,
    Query(q): Query<MessagesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.chat_client.clone();
    let reply = client
        .get_messages(GetMessagesRequest {
            channel_type,
            channel_id: q.channel_id.map(|id| id.to_string()).unwrap_or_default(), // Uuid → String at gRPC boundary; "" = global
            limit: q.limit.unwrap_or(MESSAGES_DEFAULT_PAGE_SIZE),
            before_id: q.before_id.map(|id| id.to_string()).unwrap_or_default(),   // Uuid → String; "" = from latest
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    let messages: Vec<serde_json::Value> = reply.messages.into_iter().map(|m| {
        serde_json::json!({
            "id": m.id,
            "sender_id": m.sender_id,
            "sender_name": m.sender_name,
            "content": m.content,
            "created_at": m.created_at,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "messages": messages })))
}
