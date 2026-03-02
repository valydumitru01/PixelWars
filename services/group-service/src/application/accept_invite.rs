use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::{commands::AcceptInviteCommand, MAX_GROUP_SIZE};
use crate::ports::{EventPublisher, GroupRepository, InviteRepository};

pub struct AcceptInviteOutput {
    pub group_id: Uuid,
}

pub struct AcceptInvite {
    group_repo: Arc<dyn GroupRepository>,
    invite_repo: Arc<dyn InviteRepository>,
    events: Arc<dyn EventPublisher>,
}

impl AcceptInvite {
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

    pub async fn execute(&self, cmd: AcceptInviteCommand) -> Result<AcceptInviteOutput, AppError> {
        // 1. Find and validate the invite
        let invite = self.invite_repo
            .find_by_id(cmd.invite_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Invite not found".into()))?;

        // 2. Check that the invite is for the accepting user
        if invite.to_user != cmd.user_id {
            return Err(AppError::Forbidden(
                "This invite is not for you".into(),
            ));
        }

        // 3. Check invite is still pending
        if invite.status != "pending" {
            return Err(AppError::Conflict(
                "Invite is no longer pending".into(),
            ));
        }

        // 4. Re-check group capacity before adding
        let member_count = self.group_repo
            .get_member_count(invite.group_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if member_count >= MAX_GROUP_SIZE {
            return Err(AppError::RateLimited(
                "Group is at maximum capacity".into(),
            ));
        }

        // 5. Update invite status
        self.invite_repo
            .accept(cmd.invite_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 6. Add user to group
        self.group_repo
            .add_member(invite.group_id, cmd.user_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 7. Publish event (best-effort)
        if let Err(e) = self.events
            .invite_accepted(invite.group_id, cmd.user_id)
            .await
        {
            tracing::warn!(error = %e, "Failed to publish GroupInviteAccepted event");
        }

        info!(group_id = %invite.group_id, user_id = %cmd.user_id, "Invite accepted");

        Ok(AcceptInviteOutput {
            group_id: invite.group_id,
        })
    }
}
