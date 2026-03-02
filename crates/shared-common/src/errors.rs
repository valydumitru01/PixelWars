use serde_json::to_string;

#[derive(thiserror::Error, Debug)]
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
    RateLimited(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Messaging error: {0}")]
    Messaging(String),

    #[error("gRPC error: {0}")]
    Grpc(String),
}

pub fn app_error_to_status(err: AppError) -> tonic::Status {
    use tonic::Status;
    match err {
        AppError::AuthError(msg) => Status::unauthenticated(msg),
        AppError::NotFound(msg) => Status::not_found(msg),
        AppError::Validation(msg) => Status::invalid_argument(msg),
        AppError::Conflict(msg) => Status::already_exists(msg),
        AppError::Forbidden(msg) => Status::permission_denied(msg),
        AppError::Internal(msg) => Status::internal(msg),
        AppError::Database(msg) => Status::internal(msg),
        AppError::Messaging(msg) => Status::internal(msg),
        AppError::Grpc(msg) => Status::internal(msg),
        AppError::RateLimited(msg) => Status::resource_exhausted(msg),
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, msg) = match &self {
            AppError::AuthError(_)  => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound(_)   => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Conflict(_)   => (StatusCode::CONFLICT, self.to_string()),
            AppError::Forbidden(_)  => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::RateLimited(_)   => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            _                       => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, axum::Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
