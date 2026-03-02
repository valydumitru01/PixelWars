use crate::state::AppState;
use anyhow::{anyhow, Context};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum::extract::ConnectInfo;
use tower::ServiceExt;
use shared_common::errors::AppError;

pub async fn rate_limit(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> { // Return AppError directly
    let user_ip = request
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .ok_or_else(|| AppError::Internal("Failed to extract IP".to_string()))?;

    // Logic for Redis check here...
    let is_allowed = true;

    if is_allowed {
        Ok(next.run(request).await)
    } else {
        Err(AppError::RateLimited(format!("Rate limit exceeded for IP: {}", user_ip)))
    }
}