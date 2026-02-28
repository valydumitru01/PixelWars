use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::info_span;

/// Middleware that creates a tracing span for each request.
pub async fn request_tracing(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let span = info_span!(
        "http_request",
        method = %method,
        uri = %uri,
    );

    let _guard = span.enter();
    next.run(request).await
}
