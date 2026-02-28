use axum::http::StatusCode;

/// Validate an existing session / refresh tokens.
pub async fn validate_session() -> Result<&'static str, StatusCode> {
    // TODO: Validate JWT, refresh if near expiry
    Ok("session_valid")
}
