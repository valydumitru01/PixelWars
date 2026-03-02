use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::{commands::SendInviteCommand, MAX_GROUP_SIZE};
use crate::ports::{EventPublisher, GroupRepository, InviteRepository};

pub struct SendInviteOutput {
    pub invite_id: Uuid,
}

pub struct SendInvite {
    group_repo: Arc<dyn GroupRepository>,
    invite_repo: Arc<dyn InviteRepository>,
    events: Arc<dyn EventPublisher>,
}

impl SendInvite {
    pub fn new(
        group_repo: Arc<dyn GroupRepository>,
        invite_repo: Arc<dyn InviteRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            group_repo,
            invite_repo,
            events,
        }
    }

    pub async fn execute(&self, cmd: SendInviteCommand) -> Result<SendInviteOutput, AppError> {
        // 1. Check sender is a member of the group
        let member_ids = self.group_repo
            .get_member_ids(cmd.group_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if !member_ids.contains(&cmd.from_user) {
            return Err(AppError::Forbidden(
                "You are not a member of this group".into(),
            ));
        }

        // 2. Check group size < MAX_GROUP_SIZE
        let member_count = self.group_repo
            .get_member_count(cmd.group_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if member_count >= MAX_GROUP_SIZE {
            return Err(AppError::RateLimited(
                "Group is at maximum capacity".into(),
            ));
        }

        // 3. Create the invite
        let invite_id = Uuid::new_v4();
        self.invite_repo
            .create(invite_id, cmd.group_id, cmd.from_user, cmd.to_user)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 4. Publish event (best-effort)
        if let Err(e) = self.events
            .invite_sent(cmd.group_id, cmd.from_user, cmd.to_user)
            .await
        {
            tracing::warn!(error = %e, "Failed to publish GroupInviteSent event");
        }

        info!(invite_id = %invite_id, from = %cmd.from_user, to = %cmd.to_user, "Group invite sent");

        Ok(SendInviteOutput { invite_id })
    }
}
