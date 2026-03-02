use tonic::{Request, Response, Status};

use crate::domain::commands::{CastVoteCommand, GetResultsQuery};
use crate::state::VotingState;

pub mod proto {
    tonic::include_proto!("pixelwar.voting");
}

use proto::voting_service_server::VotingService;
pub use proto::voting_service_server::VotingServiceServer;
use proto::*;

pub struct VotingGrpcService {
    pub state: VotingState,
}

#[tonic::async_trait]
impl VotingService for VotingGrpcService {
    async fn cast_vote(
        &self,
        request: Request<CastVoteRequest>,
    ) -> Result<Response<CastVoteReply>, Status> {
        let req = request.into_inner();
        let round_id: uuid::Uuid = req.round_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid round_id"))?;
        let voter_id: uuid::Uuid = req.voter_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid voter_id"))?;
        let target_id: uuid::Uuid = req.target_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid target_id"))?;

        let cmd = CastVoteCommand {
            round_id,
            voter_id,
            target_id,
            target_type: String::new(), // Will be determined by the use case based on target existence
        };

        let output = self.state.cast_vote
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(CastVoteReply {
            vote_id: output.vote_id.to_string(),
        }))
    }

    async fn get_results(
        &self,
        request: Request<GetResultsRequest>,
    ) -> Result<Response<GetResultsReply>, Status> {
        let round_id: uuid::Uuid = request.into_inner().round_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid round_id"))?;

        let query = GetResultsQuery { round_id };

        let results = self.state.get_results
            .execute(query)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        let rankings: Vec<VoteRankingProto> = results
            .rankings
            .into_iter()
            .map(|ranking| VoteRankingProto {
                target_id: ranking.target_id.to_string(),
                vote_count: ranking.vote_count,
                rank: ranking.rank,
            })
            .collect();

        Ok(Response::new(GetResultsReply {
            round_id: round_id.to_string(),
            rankings,
        }))
    }

    async fn is_voting_open(
        &self,
        request: Request<IsVotingOpenRequest>,
    ) -> Result<Response<IsVotingOpenReply>, Status> {
        let round_id: uuid::Uuid = request.into_inner().round_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid round_id"))?;

        let is_open = self.state.is_voting_open
            .execute(round_id)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(IsVotingOpenReply { is_open }))
    }
}

