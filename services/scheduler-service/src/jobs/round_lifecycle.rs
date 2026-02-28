use shared_common::events::DomainEvent;
use shared_messaging::events::subjects;
use tracing::info;
use uuid::Uuid;

use crate::state::SchedulerState;

/// Start a new round (drawing period lasts 1 month).
pub async fn start_round(state: &SchedulerState) -> anyhow::Result<Uuid> {
    let round_id = Uuid::new_v4();

    // TODO: Create round record in PostgreSQL
    // TODO: Reset canvas state
    // TODO: Publish RoundStarted event

    let event = DomainEvent::RoundStarted { round_id };
    state
        .nats
        .publish(subjects::SCHEDULER_ROUND_STARTED, &event)
        .await?;

    info!(round_id = %round_id, "New round started");
    Ok(round_id)
}

/// End the current round.
pub async fn end_round(state: &SchedulerState, round_id: Uuid) -> anyhow::Result<()> {
    // TODO: Finalize round in PostgreSQL
    // TODO: Trigger final snapshot
    // TODO: Open voting window

    let event = DomainEvent::RoundEnded { round_id };
    state
        .nats
        .publish(subjects::SCHEDULER_ROUND_ENDED, &event)
        .await?;

    info!(round_id = %round_id, "Round ended");
    Ok(())
}
