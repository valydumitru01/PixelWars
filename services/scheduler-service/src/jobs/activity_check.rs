use tracing::info;

use crate::ports::RoundRepository;
use crate::state::SchedulerState;

/// Thin job wrapper: run activity check on the active round.
pub async fn run_activity_check_job(state: &SchedulerState) -> anyhow::Result<()> {
    match state.round_repo.get_active().await {
        Ok(Some(round)) => {
            info!(round_id = %round.id, "Running activity check on active round");
            state.run_activity_check.execute(round.id).await?;
        }
        Ok(None) => {
            info!("No active round, skipping activity check");
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get active round for activity check");
            return Err(e);
        }
    }
    Ok(())
}
