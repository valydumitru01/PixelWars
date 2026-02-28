use axum::{extract::State, http::StatusCode, Json};
use shared_common::events::DomainEvent;
use shared_common::models::{AuthResponse, RegisterRequest};
use shared_messaging::events::subjects;
use tracing::{info, error};
use uuid::Uuid;

use crate::{jwt, password, state::AuthState};

pub async fn handle_register(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let password_hash = password::hash_password(&req.password.as_str())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = Uuid::new_v4();

    // TODO: Insert user into PostgreSQL
    // sqlx::query!("INSERT INTO users ...")

    let token = jwt::create_token(user_id, &req.username.as_str(), &state.jwt_secret.as_str())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Publish event
    let event = DomainEvent::UserRegistered {
        user_id,
        username: req.username.clone(),
    };
    if let Err(e) = state.nats.publish(subjects::AUTH_USER_REGISTERED, &event).await {
        error!(error = %e, "Failed to publish UserRegistered event");
    }

    info!(user_id = %user_id, username = %req.username, "User registered");

    Ok(Json(AuthResponse {
        token,
        user_id,
        username: req.username,
    }))
}
