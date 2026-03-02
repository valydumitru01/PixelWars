use std::sync::Arc;

use shared_common::errors::AppError;
use uuid::Uuid;

use crate::ports::GroupRepository;

pub struct GetGroupOutput {
    pub group_id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub member_ids: Vec<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetGroup {
    group_repo: Arc<dyn GroupRepository>,
}

impl GetGroup {
    pub fn new(group_repo: Arc<dyn GroupRepository>) -> Self {
        Self { group_repo }
    }

    pub async fn execute(&self, group_id: Uuid) -> Result<GetGroupOutput, AppError> {
        // 1. Find the group
        let group = self.group_repo
            .find_by_id(group_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Group not found".into()))?;

        // 2. Fetch members
        let member_ids = self.group_repo
            .get_member_ids(group_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(GetGroupOutput {
            group_id: group.id,
            name: group.name,
            creator_id: group.creator_id,
            member_ids,
            created_at: group.created_at,
        })
    }
}
