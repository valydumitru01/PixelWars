use axum::{extract::State, http::StatusCode, Json};
use shared_common::models::{AuthResponse, LoginRequest};
use tracing::info;
use uuid::Uuid;

use crate::{jwt, state::AuthState};

pub async fn handle_login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // TODO: Look up user by email in PostgreSQL
    // TODO: Verify password with password::verify_password

    let user_id = Uuid::new_v4(); // placeholder
    let username = "placeholder".to_string();

    let token = jwt::create_token(user_id, &username, &state.jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(user_id = %user_id, "User logged in");

    Ok(Json(AuthResponse {
        token,
        user_id,
        username,
    }))
}
