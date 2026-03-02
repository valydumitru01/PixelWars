use shared_common::models::vote::{VoteRanking, VoteResults};
use uuid::Uuid;

/// Tally votes and produce ranked results.
pub fn tally_votes(round_id: Uuid, votes: &[(Uuid, u64)]) -> VoteResults {
    let mut sorted: Vec<_> = votes.to_vec();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let rankings = sorted
        .iter()
        .enumerate()
        .map(|(i, (target_id, count))| VoteRanking {
            target_id: *target_id,
            vote_count: *count,
            rank: (i + 1) as u32,
        })
        .collect();

    VoteResults { round_id, rankings }
}
