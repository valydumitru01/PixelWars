use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};
use shared_common::{errors::AppError, models::user::UserClaims};
use crate::clients::groups::*;
use crate::state::AppState;
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_group))
        .route("/{id}", get(get_group))
        .route("/{id}/invite", post(invite_member))
        .route("/{id}/invite/accept", post(accept_invite))
}

#[derive(serde::Deserialize)]
pub struct CreateGroupBody {
    pub name: String,
    pub round_id: Uuid,
}

async fn create_group(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(body): Json<CreateGroupBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.group_client.clone();
    let reply = client
        .create_group(CreateGroupRequest {
            name: body.name,
            creator_id: claims.sub.to_string(),    // Uuid → String at gRPC boundary
            round_id: body.round_id.to_string(),   // Uuid → String at gRPC boundary
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({
        "group_id": reply.group_id,
        "name": reply.name,
        "member_ids": reply.member_ids,
    })))
}

async fn get_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.group_client.clone();
    let reply = client
        .get_group(GetGroupRequest { group_id: id.to_string() })  // Uuid → String at gRPC boundary
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({
        "group_id": reply.group_id,
        "name": reply.name,
        "creator_id": reply.creator_id,
        "member_ids": reply.member_ids,
    })))
}

#[derive(serde::Deserialize)]
pub struct InviteBody {
    pub to_user_id: Uuid,
}

async fn invite_member(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<UserClaims>,
    Json(body): Json<InviteBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.group_client.clone();
    let reply = client
        .send_invite(SendInviteRequest {
            group_id: id.to_string(),                  // Uuid → String at gRPC boundary
            from_user_id: claims.sub.to_string(),      // Uuid → String at gRPC boundary
            to_user_id: body.to_user_id.to_string(),   // Uuid → String at gRPC boundary
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({ "invite_id": reply.invite_id })))
}

#[derive(serde::Deserialize)]
pub struct AcceptInviteBody {
    pub invite_id: Uuid,
}

async fn accept_invite(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(body): Json<AcceptInviteBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.group_client.clone();
    let reply = client
        .accept_invite(AcceptInviteRequest {
            invite_id: body.invite_id.to_string(),  // Uuid → String at gRPC boundary
            user_id: claims.sub.to_string(),         // Uuid → String at gRPC boundary
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({
        "group_id": reply.group_id,
        "member_ids": reply.member_ids,
    })))
}
