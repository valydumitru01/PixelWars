use axum::{extract::State, Json};
use shared_common::{errors::AppError, models::user::{AuthResponse, LoginRequest}};
use tracing::info;

use crate::{jwt, password, state::AuthState};

pub async fn handle_login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query!(
        "SELECT id, username, password_hash, is_disqualified
         FROM users WHERE email = $1 AND is_active = true",
        req.email
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::AuthError("Invalid email or password".into()))?;

    if user.is_disqualified {
        return Err(AppError::Forbidden("Account has been disqualified".into()));
    }

    let valid = password::verify_password(&req.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::AuthError("Invalid email or password".into()));
    }

    let token = jwt::create_token(user.id, &user.username, &state.jwt_secret)?;

    info!(user_id = %user.id, "User logged in");

    Ok(Json(AuthResponse { token, user_id: user.id, username: user.username }))
}
