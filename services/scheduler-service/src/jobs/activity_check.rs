use chrono::Utc;
use shared_common::events::DomainEvent;
use shared_messaging::events::subjects;
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::SchedulerState;

/// Check all active users and disqualify those who haven't drawn in 3 days.
pub async fn run_activity_check(state: &SchedulerState, round_id: Uuid) -> anyhow::Result<()> {
    let three_days_ago = Utc::now() - chrono::Duration::days(3);

    // TODO: Query PostgreSQL for users whose last_draw_at < three_days_ago
    // For each inactive user:
    //   1. Mark as disqualified in DB
    //   2. Publish UserDisqualified event
    //   3. Release their parcel

    info!(
        round_id = %round_id,
        cutoff = %three_days_ago,
        "Activity check completed"
    );

    Ok(())
}
