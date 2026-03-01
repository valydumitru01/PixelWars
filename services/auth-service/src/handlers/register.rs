use axum::{extract::State, Json};
use shared_common::{errors::AppError, models::user::{AuthResponse, RegisterRequest}};
use tracing::info;

use crate::{jwt, password, state::AuthState};

pub async fn handle_register(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate input
    if req.username.len() < 3 {
        return Err(AppError::Validation("Username must be at least 3 characters".into()));
    }
    if req.password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".into()));
    }

    // Check duplicate
    let exists = sqlx::query!(
        "SELECT id FROM users WHERE email = $1 OR username = $2",
        req.email,
        req.username
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if exists.is_some() {
        return Err(AppError::Conflict("Username or email already taken".into()));
    }

    let password_hash = password::hash_password(&req.password)?;
    let user_id = uuid::Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)",
        user_id,
        req.username,
        req.email,
        password_hash
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let token = jwt::create_token(user_id, &req.username, &state.jwt_secret)?;

    info!(user_id = %user_id, username = %req.username, "User registered");

    Ok(Json(AuthResponse { token, user_id, username: req.username }))
}
