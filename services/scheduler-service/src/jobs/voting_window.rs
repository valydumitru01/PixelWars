use crate::state::SchedulerState;

/// Thin job wrapper: close expired voting windows.
pub async fn close_expired_voting_job(state: &SchedulerState) -> anyhow::Result<()> {
    state.close_voting.execute().await
}
