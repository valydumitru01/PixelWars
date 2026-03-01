use chrono::{Duration, Utc};
use shared_common::events::DomainEvent;
use shared_messaging::events::subjects;
use tracing::info;
use uuid::Uuid;

use crate::state::SchedulerState;

/// Start a new 1-month round.
pub async fn start_round(state: &SchedulerState) -> anyhow::Result<Uuid> {
    // Deactivate any existing active round
    sqlx::query!("UPDATE rounds SET is_active = false WHERE is_active = true")
        .execute(&state.db)
        .await?;

    // Reset all users' disqualification status
    sqlx::query!("UPDATE users SET is_disqualified = false, last_draw_at = NULL WHERE is_active = true")
        .execute(&state.db)
        .await?;

    let round_id = Uuid::new_v4();
    let now = Utc::now();
    let ends_at = now + Duration::days(30);

    sqlx::query!(
        "INSERT INTO rounds (id, started_at, ends_at, is_active) VALUES ($1, $2, $3, true)",
        round_id, now, ends_at
    )
    .execute(&state.db)
    .await?;

    let event = DomainEvent::RoundStarted { round_id };
    state.nats.publish(subjects::SCHEDULER_ROUND_STARTED, &event).await?;

    info!(round_id = %round_id, ends_at = %ends_at, "New round started");
    Ok(round_id)
}

/// End the current round and open voting.
pub async fn end_round(state: &SchedulerState, round_id: Uuid) -> anyhow::Result<()> {
    let now = Utc::now();
    let voting_ends = now + Duration::days(3);

    sqlx::query!(
        "UPDATE rounds SET is_active = false, voting_starts_at = $1, voting_ends_at = $2 WHERE id = $3",
        now, voting_ends, round_id
    )
    .execute(&state.db)
    .await?;

    let event = DomainEvent::RoundEnded { round_id };
    state.nats.publish(subjects::SCHEDULER_ROUND_ENDED, &event).await?;

    let vote_event = DomainEvent::VotingWindowOpened { round_id };
    state.nats.publish(subjects::VOTING_WINDOW_OPENED, &vote_event).await?;

    info!(round_id = %round_id, voting_until = %voting_ends, "Round ended, voting opened");
    Ok(())
}

/// Get the currently active round, if any.
pub async fn get_active_round(state: &SchedulerState) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query!("SELECT id FROM rounds WHERE is_active = true LIMIT 1")
        .fetch_optional(&state.db)
        .await?;
    Ok(row.map(|r| r.id))
}
