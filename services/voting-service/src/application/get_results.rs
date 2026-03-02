use std::sync::Arc;

use shared_common::errors::AppError;
use shared_common::models::vote::VoteResults;

use crate::domain::commands::GetResultsQuery;
use crate::domain::tallying;
use crate::ports::VoteRepository;

pub struct GetResults {
    vote_repo: Arc<dyn VoteRepository>,
}

impl GetResults {
    pub fn new(vote_repo: Arc<dyn VoteRepository>) -> Self {
        Self { vote_repo }
    }

    pub async fn execute(&self, query: GetResultsQuery) -> Result<VoteResults, AppError> {
        // 1. Aggregate votes from repository
        let aggregates = self
            .vote_repo
            .aggregate_by_round(query.round_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 2. Convert to (target_id, vote_count) tuples
        let votes: Vec<_> = aggregates
            .into_iter()
            .map(|agg| (agg.target_id, agg.vote_count as u64))
            .collect();

        // 3. Tally using domain logic
        let results = tallying::tally_votes(query.round_id, &votes);

        Ok(results)
    }
}
