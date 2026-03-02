use std::sync::Arc;

use shared_common::errors::AppError;
use uuid::Uuid;

use crate::ports::GroupRepository;

pub struct GetUserGroupOutput {
    pub group_id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub member_ids: Vec<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetUserGroup {
    group_repo: Arc<dyn GroupRepository>,
}

impl GetUserGroup {
    pub fn new(group_repo: Arc<dyn GroupRepository>) -> Self {
        Self { group_repo }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        round_id: Uuid,
    ) -> Result<GetUserGroupOutput, AppError> {
        // 1. Find user's group in the given round
        let group = self.group_repo
            .find_user_group(user_id, round_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User is not in a group this round".into()))?;

        // 2. Fetch members
        let member_ids = self.group_repo
            .get_member_ids(group.id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(GetUserGroupOutput {
            group_id: group.id,
            name: group.name,
            creator_id: group.creator_id,
            member_ids,
            created_at: group.created_at,
        })
    }
}
