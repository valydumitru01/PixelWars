use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use shared_common::models::user::UserClaims;

/// Middleware that validates JWT and injects UserClaims into request extensions.
pub async fn require_auth(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If the secret extension is absent the router was misconfigured — fail
    // closed rather than silently validating tokens with an empty key.
    let jwt_secret = request
        .extensions()
        .get::<String>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = decode::<UserClaims>(
        auth_header,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    // Make the claims available to handlers
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
