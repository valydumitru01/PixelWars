use std::sync::Arc;

use shared_common::models::user::UserClaims;

use crate::ports::TokenProvider;

pub struct ValidateToken {
    tokens: Arc<dyn TokenProvider>,
}

impl ValidateToken {
    pub fn new(tokens: Arc<dyn TokenProvider>) -> Self {
        Self { tokens }
    }

    /// Decode and validate a JWT. Returns `Ok(claims)` or `Err` if invalid/expired.
    pub fn execute(&self, token: &str) -> anyhow::Result<UserClaims> {
        self.tokens.validate_token(token)
    }
}
