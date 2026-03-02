use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::commands::LoginCommand;
use crate::ports::{EventPublisher, PasswordHasher, TokenProvider, UserRepository};

/// Output returned after a successful login.
pub struct LoginOutput {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
}

pub struct LoginUser {
    user_repo: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenProvider>,
    events: Arc<dyn EventPublisher>,
}

impl LoginUser {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenProvider>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { user_repo, hasher, tokens, events }
    }

    pub async fn execute(&self, cmd: LoginCommand) -> Result<LoginOutput, AppError> {
        // 1. Find user by email
        let user = self.user_repo
            .find_by_email(&cmd.email)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::AuthError("Invalid email or password".into()))?;

        // 2. Check disqualified
        if user.is_disqualified {
            return Err(AppError::Forbidden("Account has been disqualified".into()));
        }

        // 3. Verify password
        let valid = self.hasher
            .verify(&cmd.password, &user.password_hash)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if !valid {
            return Err(AppError::AuthError("Invalid email or password".into()));
        }

        // 4. Generate token
        let token = self.tokens
            .create_token(user.id, &user.username)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // 5. Publish event (best-effort)
        if let Err(e) = self.events.user_logged_in(user.id).await {
            tracing::warn!(error = %e, "Failed to publish UserLoggedIn event");
        }

        info!(user_id = %user.id, "User logged in");

        Ok(LoginOutput { token, user_id: user.id, username: user.username })
    }
}
