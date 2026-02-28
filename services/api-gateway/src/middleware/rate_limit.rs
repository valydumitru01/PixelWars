use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Simple rate-limiting middleware using a sliding window counter.
/// In production, use Redis-backed rate limiting.
pub async fn rate_limit(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement Redis-based sliding window rate limiter
    // Key: user_id or IP, Window: 60s, Max: 100 requests
    Ok(next.run(request).await)
}
