use shared_common::errors::AppError;

const MIN_USERNAME_LEN: usize = 3;
const MIN_PASSWORD_LEN: usize = 8;
const MAX_DESCRIPTION_LEN: usize = 500;

pub fn validate_username(username: &str) -> Result<(), AppError> {
    if username.len() < MIN_USERNAME_LEN {
        return Err(AppError::Validation(format!(
            "Username must be at least {MIN_USERNAME_LEN} characters"
        )));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(AppError::Validation(format!(
            "Password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    Ok(())
}
