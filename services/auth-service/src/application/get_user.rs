use std::sync::Arc;

use shared_common::errors::AppError;
use uuid::Uuid;

use crate::ports::{UserRepository, UserRow};

pub struct GetUser {
    user_repo: Arc<dyn UserRepository>,
}

impl GetUser {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<UserRow, AppError> {
        self.user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".into()))
    }
}
