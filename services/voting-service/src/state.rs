use std::sync::Arc;

use crate::application::{cast_vote, get_results, is_voting_open};

#[derive(Clone)]
pub struct VotingState {
    pub cast_vote: Arc<cast_vote::CastVote>,
    pub get_results: Arc<get_results::GetResults>,
    pub is_voting_open: Arc<is_voting_open::IsVotingOpen>,
}
