use chrono::Utc;
use shared_common::events::DomainEvent;
use shared_messaging::events::subjects;
use tracing::info;

use crate::state::SchedulerState;

/// Close any expired voting windows.
pub async fn close_expired_voting(state: &SchedulerState) -> anyhow::Result<()> {
    let now = Utc::now();

    let expired = sqlx::query!(
        "SELECT id FROM rounds WHERE voting_ends_at IS NOT NULL AND voting_ends_at < $1 AND is_active = false",
        now
    )
    .fetch_all(&state.db)
    .await?;

    for round in &expired {
        let event = DomainEvent::VotingWindowClosed { round_id: round.id };
        let _ = state.nats.publish(subjects::VOTING_WINDOW_CLOSED, &event).await;
        info!(round_id = %round.id, "Voting window closed");
    }

    Ok(())
}
