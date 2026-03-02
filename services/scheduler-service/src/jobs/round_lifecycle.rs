use tracing::info;

use crate::ports::RoundRepository;
use crate::state::SchedulerState;

/// Thin job wrapper: check for expired rounds and end them.
pub async fn end_expired_round_job(state: &SchedulerState) -> anyhow::Result<()> {
    match state.round_repo.get_active().await {
        Ok(Some(round)) => {
            // Check if the round has expired
            if chrono::Utc::now() > round.ends_at {
                info!(round_id = %round.id, "Round expired, ending it...");
                state.end_round.execute(round.id).await?;
            }
        }
        Ok(None) => {
            info!("No active round to expire");
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to check for expired round");
            return Err(e);
        }
    }
    Ok(())
}
