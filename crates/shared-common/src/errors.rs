use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Messaging error: {0}")]
    Messaging(String),

    #[error("gRPC error: {0}")]
    Grpc(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::AuthError(_)  => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound(_)   => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Conflict(_)   => (StatusCode::CONFLICT, self.to_string()),
            AppError::Forbidden(_)  => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::RateLimited   => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            _                       => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
