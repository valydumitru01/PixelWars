use axum::http::StatusCode;
use axum::Json;
use serde_json::json;
use thiserror::Error;
use axum::response::{IntoResponse, Response};

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

    #[error("Rate limited")]
    RateLimited,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Messaging error: {0}")]
    Messaging(String),
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED", "Too many requests".to_string()),
            // We hide the exact details of Internal/DB/Messaging errors from the user for security
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Internal database error".to_string()),
            AppError::Messaging(_) => (StatusCode::INTERNAL_SERVER_ERROR, "MESSAGING_ERROR", "Internal messaging error".to_string()),
        };

        // Log the actual error for your Jaeger/Terminal observability
        tracing::error!(status = %status, error_type = %error_type, message = %message);

        let body = Json(json!({
            "status": "error",
            "type": error_type,
            "message": message
        }));

        (status, body).into_response()
    }
}


impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}