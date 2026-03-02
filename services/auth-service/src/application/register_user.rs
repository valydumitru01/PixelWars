use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::commands::RegisterCommand;
use crate::domain::validation;
use crate::ports::{EventPublisher, PasswordHasher, TokenProvider, UserRepository};

/// Output returned after a successful registration.
pub struct RegisterOutput {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
}

pub struct RegisterUser {
    user_repo: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenProvider>,
    events: Arc<dyn EventPublisher>,
}

impl RegisterUser {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenProvider>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { user_repo, hasher, tokens, events }
    }

    pub async fn execute(&self, cmd: RegisterCommand) -> Result<RegisterOutput, AppError> {
        // 1. Validate input
        validation::validate_username(&cmd.username)?;
        validation::validate_password(&cmd.password)?;

        // 2. Check duplicate
        let exists = self.user_repo
            .exists_by_email_or_username(&cmd.email, &cmd.username)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if exists {
            return Err(AppError::Conflict("Username or email already taken".into()));
        }

        // 3. Hash password
        let password_hash = self.hasher
            .hash(&cmd.password)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // 4. Create user
        let user_id = Uuid::new_v4();
        self.user_repo
            .create(user_id, &cmd.username, &cmd.email, &password_hash)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 5. Generate token
        let token = self.tokens
            .create_token(user_id, &cmd.username)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // 6. Publish event (best-effort — don't fail registration if NATS is down)
        if let Err(e) = self.events.user_registered(user_id, &cmd.username).await {
            tracing::warn!(error = %e, "Failed to publish UserRegistered event");
        }

        info!(user_id = %user_id, username = %cmd.username, "User registered");

        Ok(RegisterOutput { token, user_id, username: cmd.username })
    }
}
