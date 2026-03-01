use chrono::Utc;
use shared_common::events::DomainEvent;
use shared_messaging::events::subjects;
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::SchedulerState;

/// Disqualify users who haven't drawn in 3 days.
pub async fn run_activity_check(state: &SchedulerState, round_id: Uuid) -> anyhow::Result<()> {
    let cutoff = Utc::now() - chrono::Duration::days(3);

    let inactive_users = sqlx::query!(
        r#"SELECT u.id, u.username
           FROM users u
           JOIN parcels p ON p.user_id = u.id AND p.round_id = $1
           WHERE u.is_active = true
             AND u.is_disqualified = false
             AND (u.last_draw_at IS NULL OR u.last_draw_at < $2)"#,
        round_id,
        cutoff
    )
    .fetch_all(&state.db)
    .await?;

    let count = inactive_users.len();

    for user in &inactive_users {
        // Mark disqualified
        sqlx::query!(
            "UPDATE users SET is_disqualified = true WHERE id = $1",
            user.id
        )
        .execute(&state.db)
        .await?;

        // Release their parcel
        sqlx::query!(
            "DELETE FROM parcels WHERE user_id = $1 AND round_id = $2",
            user.id, round_id
        )
        .execute(&state.db)
        .await?;

        let event = DomainEvent::UserDisqualified {
            user_id: user.id,
            round_id,
            reason: "No drawing activity for 3 days".to_string(),
        };
        if let Err(e) = state.nats.publish(subjects::SCHEDULER_USER_DISQUALIFIED, &event).await {
            warn!(error = %e, user_id = %user.id, "Failed to publish disqualification event");
        }
    }

    info!(round_id = %round_id, disqualified = count, "Activity check completed");
    Ok(())
}
