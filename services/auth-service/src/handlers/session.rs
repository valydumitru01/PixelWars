use axum::{extract::State, http::HeaderMap, Json};
use shared_common::{errors::AppError, models::user::UserClaims};
use tracing::info;

use crate::{jwt, state::AuthState};

/// Validate the current token and return its claims.
pub async fn validate_session(
    State(state): State<AuthState>,
    headers: HeaderMap,
) -> Result<Json<UserClaims>, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::AuthError("Missing Authorization header".into()))?;

    let claims = jwt::validate_token(token, &state.jwt_secret)
        .map_err(|_| AppError::AuthError("Invalid or expired token".into()))?;

    Ok(Json(claims))
}
