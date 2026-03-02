use std::sync::Arc;

use crate::application::{close_voting, end_round, run_activity_check, start_round};
use crate::ports::RoundRepository;

#[derive(Clone)]
pub struct SchedulerState {
    pub run_activity_check: Arc<run_activity_check::RunActivityCheck>,
    pub end_round: Arc<end_round::EndRound>,
    pub close_voting: Arc<close_voting::CloseVoting>,
    pub start_round: Arc<start_round::StartRound>,
    pub round_repo: Arc<dyn RoundRepository>,
}
