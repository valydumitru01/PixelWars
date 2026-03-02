use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::commands::CreateGroupCommand;
use crate::ports::{EventPublisher, GroupRepository};

pub struct CreateGroupOutput {
    pub group_id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub member_ids: Vec<Uuid>,
}

pub struct CreateGroup {
    group_repo: Arc<dyn GroupRepository>,
    events: Arc<dyn EventPublisher>,
}

impl CreateGroup {
    pub fn new(
        group_repo: Arc<dyn GroupRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { group_repo, events }
    }

    pub async fn execute(&self, cmd: CreateGroupCommand) -> Result<CreateGroupOutput, AppError> {
        // 1. Check user has a parcel this round
        let has_parcel = self.group_repo
            .user_has_parcel(cmd.creator_id, cmd.round_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if !has_parcel {
            return Err(AppError::Forbidden(
                "Must claim a parcel before creating a group".into(),
            ));
        }

        // 2. Check user is not already in a group this round
        let existing_group = self.group_repo
            .find_user_group(cmd.creator_id, cmd.round_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if existing_group.is_some() {
            return Err(AppError::Conflict(
                "Already in a group this round".into(),
            ));
        }

        // 3. Create the group
        let group_id = Uuid::new_v4();
        self.group_repo
            .create(group_id, &cmd.name, cmd.creator_id, cmd.round_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 4. Add creator as first member
        self.group_repo
            .add_member(group_id, cmd.creator_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 5. Publish event (best-effort)
        if let Err(e) = self.events.group_created(group_id, cmd.creator_id).await {
            tracing::warn!(error = %e, "Failed to publish GroupCreated event");
        }

        info!(group_id = %group_id, name = %cmd.name, "Group created");

        Ok(CreateGroupOutput {
            group_id,
            name: cmd.name,
            creator_id: cmd.creator_id,
            member_ids: vec![cmd.creator_id],
        })
    }
}
