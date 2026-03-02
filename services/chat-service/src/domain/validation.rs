use shared_common::errors::AppError;

/// Validate message content (1-2000 characters).
pub fn validate_content(content: &str) -> Result<(), AppError> {
    let len = content.len();
    if len == 0 || len > 2000 {
        return Err(AppError::Validation(
            "Message content must be 1-2000 characters".into(),
        ));
    }
    Ok(())
}
