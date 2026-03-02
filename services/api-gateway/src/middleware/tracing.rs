use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::{info_span, span};
use shared_common::models::UserClaims;

/// Middleware that creates a tracing span for each request.
pub async fn request_tracing(
    request: Request,
    next: Next,
) -> Response {
    let span = tracing::Span::current();

    if let Some(user) = request.extensions().get::<UserClaims>() {
        // 3. Record it directly onto the parent span!
        span.record("user_id", &user.sub.to_string());
        span.record("username", &user.username.as_str());
    }

    let _guard = span.enter();
    next.run(request).await
}
