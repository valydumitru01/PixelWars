use axum::{
    extract::State,
    routing::{get, post},
    Extension, Json, Router,
};
use shared_common::{errors::AppError, models::user::UserClaims};
use crate::clients::voting::*;
use crate::state::AppState;
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/vote", post(cast_vote))
        .route("/results", get(get_results))
}

#[derive(serde::Deserialize)]
pub struct VoteBody {
    pub round_id: Uuid,
    pub target_id: Uuid,  // user_id or group_id being voted for
}

async fn cast_vote(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(body): Json<VoteBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.voting_client.clone();
    let reply = client
        .cast_vote(CastVoteRequest {
            round_id: body.round_id.to_string(),   // Uuid → String at gRPC boundary
            voter_id: claims.sub.to_string(),       // Uuid → String at gRPC boundary
            target_id: body.target_id.to_string(), // Uuid → String at gRPC boundary
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(serde_json::json!({ "vote_id": reply.vote_id })))
}

#[derive(serde::Deserialize)]
pub struct ResultsQuery {
    pub round_id: Uuid,
}

async fn get_results(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<ResultsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.voting_client.clone();
    let reply = client
        .get_results(GetResultsRequest { round_id: q.round_id.to_string() })  // Uuid → String at gRPC boundary
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    let rankings: Vec<serde_json::Value> = reply.rankings.into_iter().map(|r| {
        serde_json::json!({
            "target_id": r.target_id,
            "vote_count": r.vote_count,
            "rank": r.rank,
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "round_id": reply.round_id,
        "rankings": rankings,
    })))
}
