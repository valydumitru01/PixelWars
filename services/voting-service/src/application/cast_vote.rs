use std::sync::Arc;

use shared_common::errors::AppError;
use tracing::info;
use uuid::Uuid;

use crate::domain::commands::CastVoteCommand;
use crate::ports::{EventPublisher, RoundRepository, VoteRepository};

/// Output returned after a successful vote.
pub struct CastVoteOutput {
    pub vote_id: Uuid,
}

pub struct CastVote {
    vote_repo: Arc<dyn VoteRepository>,
    round_repo: Arc<dyn RoundRepository>,
    events: Arc<dyn EventPublisher>,
}

impl CastVote {
    pub fn new(
        vote_repo: Arc<dyn VoteRepository>,
        round_repo: Arc<dyn RoundRepository>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            vote_repo,
            round_repo,
            events,
        }
    }

    pub async fn execute(&self, cmd: CastVoteCommand) -> Result<CastVoteOutput, AppError> {
        // 1. Determine target_type if not provided
        let target_type = if !cmd.target_type.is_empty() {
            // Validate provided target_type
            if cmd.target_type != "parcel" && cmd.target_type != "group" {
                return Err(AppError::Validation(
                    "target_type must be 'parcel' or 'group'".into(),
                ));
            }
            cmd.target_type.clone()
        } else {
            // Auto-detect: check if target exists as a group
            let is_group = self
                .round_repo
                .target_is_group(cmd.target_id)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            if is_group {
                "group".to_string()
            } else {
                "parcel".to_string()
            }
        };

        // 2. Check voting window is open
        let voting_window = self
            .round_repo
            .get_voting_window(cmd.round_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Round not found".into()))?;

        let now = chrono::Utc::now();
        match (voting_window.voting_starts_at, voting_window.voting_ends_at) {
            (Some(start), Some(end)) => {
                if now < start || now > end {
                    return Err(AppError::Forbidden(
                        "Voting window is not open".into(),
                    ));
                }
            }
            _ => {
                return Err(AppError::Forbidden(
                    "Voting has not been scheduled for this round".into(),
                ));
            }
        }

        // 3. Prevent self-vote
        if target_type == "parcel" {
            let owns_parcel = self
                .round_repo
                .user_owns_parcel(cmd.voter_id, cmd.target_id)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            if owns_parcel {
                return Err(AppError::Forbidden(
                    "Cannot vote for your own parcel".into(),
                ));
            }
        } else {
            // For groups, prevent voting if user is a member
            let is_member = self
                .round_repo
                .user_is_group_member(cmd.voter_id, cmd.target_id)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            if is_member {
                return Err(AppError::Forbidden(
                    "Cannot vote for your own group".into(),
                ));
            }
        }

        // 4. Check if already voted this round
        let existing = self
            .vote_repo
            .find_by_round_voter(cmd.round_id, cmd.voter_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if existing.is_some() {
            return Err(AppError::Conflict(
                "You have already voted this round".into(),
            ));
        }

        // 5. Create vote
        let vote_id = Uuid::new_v4();
        self.vote_repo
            .create(
                vote_id,
                cmd.round_id,
                cmd.voter_id,
                cmd.target_id,
                &target_type,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 6. Publish event (best-effort — don't fail if NATS is down)
        if let Err(e) = self.events.vote_cast(cmd.voter_id, cmd.target_id).await {
            tracing::warn!(error = %e, "Failed to publish VoteCast event");
        }

        info!(vote_id = %vote_id, voter = %cmd.voter_id, target = %cmd.target_id, "Vote cast");

        Ok(CastVoteOutput { vote_id })
    }
}
