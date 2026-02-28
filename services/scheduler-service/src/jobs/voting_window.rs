use shared_common::events::DomainEvent;
use shared_messaging::events::subjects;
use tracing::info;
use uuid::Uuid;

use crate::state::SchedulerState;

/// Open the voting window (3 days after the drawing round ends).
pub async fn open_voting(state: &SchedulerState, round_id: Uuid) -> anyhow::Result<()> {
    let event = DomainEvent::VotingWindowOpened { round_id };
    state
        .nats
        .publish(subjects::VOTING_WINDOW_OPENED, &event)
        .await?;

    info!(round_id = %round_id, "Voting window opened");
    Ok(())
}

/// Close voting and trigger result tallying.
pub async fn close_voting(state: &SchedulerState, round_id: Uuid) -> anyhow::Result<()> {
    let event = DomainEvent::VotingWindowClosed { round_id };
    state
        .nats
        .publish(subjects::VOTING_WINDOW_CLOSED, &event)
        .await?;

    info!(round_id = %round_id, "Voting window closed");
    Ok(())
}
