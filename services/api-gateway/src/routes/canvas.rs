use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};
use shared_common::{errors::AppError, models::user::UserClaims};
use shared_common::models::PixelUpdate;
use crate::clients::canvas::*;
use crate::state::AppState;
use uuid::Uuid;
use crate::clients;

// ---------------------------------------------------------------------------
// API-level defaults for snapshot query parameters.
// These define what the HTTP API returns when the client omits a query param.
// The canvas-service enforces its own hard caps independently.
// ---------------------------------------------------------------------------
const SNAPSHOT_DEFAULT_ORIGIN: u32 = 0;
const SNAPSHOT_DEFAULT_SIZE: u32 = 256;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/parcels", post(claim_parcel))
        .route("/parcels/{id}", get(get_parcel))
        .route("/pixels", post(update_pixels))
        .route("/snapshot", get(get_snapshot))
}

#[derive(serde::Deserialize)]
pub struct ClaimBody {
    pub round_id: Uuid,
    pub origin_x: u32,
    pub origin_y: u32,
    pub width: u32,
    pub height: u32,
    pub description: String,
}

async fn claim_parcel(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(body): Json<ClaimBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.canvas_client.clone();
    let reply = client
        .claim_parcel(ClaimParcelRequest {
            user_id: claims.sub.to_string(),      // Uuid → String at gRPC boundary
            round_id: body.round_id.to_string(),  // Uuid → String at gRPC boundary
            origin_x: body.origin_x,
            origin_y: body.origin_y,
            width: body.width,
            height: body.height,
            description: body.description,
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({
        "parcel_id": reply.parcel_id,
        "origin_x": reply.origin_x,
        "origin_y": reply.origin_y,
        "width": reply.width,
        "height": reply.height,
    })))
}

async fn get_parcel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.canvas_client.clone();
    let reply = client
        .get_parcel(GetParcelRequest { parcel_id: id.to_string() })  // Uuid → String at gRPC boundary
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({
        "parcel_id": reply.parcel_id,
        "user_id": reply.user_id,
        "origin_x": reply.origin_x,
        "origin_y": reply.origin_y,
        "width": reply.width,
        "height": reply.height,
        "description": reply.description,
    })))
}

#[derive(serde::Deserialize)]
pub struct PixelUpdateBody {
    pub local_x: u32,
    pub local_y: u32,
    pub color: u32,
}

#[derive(serde::Deserialize)]
pub struct UpdatePixelsBody {
    pub parcel_id: Uuid,
    pub pixels: Vec<PixelUpdateBody>,
}

async fn update_pixels(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(body): Json<UpdatePixelsBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.canvas_client.clone();
    let reply = client
        .update_pixels(UpdatePixelsRequest {
            parcel_id: body.parcel_id.to_string(),  // Uuid → String at gRPC boundary
            user_id: claims.sub.to_string(),         // Uuid → String at gRPC boundary
            pixels: body.pixels.into_iter().map(|p| clients::canvas::PixelUpdate {
                local_x: p.local_x,
                local_y: p.local_y,
                color: p.color,
            }).collect(),
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({ "updated_count": reply.updated_count })))
}

#[derive(serde::Deserialize)]
pub struct SnapshotQuery {
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

async fn get_snapshot(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<SnapshotQuery>,
) -> Result<axum::body::Bytes, AppError> {
    let mut client = state.canvas_client.clone();
    let reply = client
        .get_snapshot(GetSnapshotRequest {
            x: q.x.unwrap_or(SNAPSHOT_DEFAULT_ORIGIN),
            y: q.y.unwrap_or(SNAPSHOT_DEFAULT_ORIGIN),
            width: q.width.unwrap_or(SNAPSHOT_DEFAULT_SIZE),
            height: q.height.unwrap_or(SNAPSHOT_DEFAULT_SIZE),
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(axum::body::Bytes::from(reply.data))
}
