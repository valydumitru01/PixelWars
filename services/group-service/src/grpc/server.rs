use tonic::{Request, Response, Status};

use crate::domain::commands::{AcceptInviteCommand, CreateGroupCommand, SendInviteCommand};
use crate::state::GroupState;

pub mod proto {
    tonic::include_proto!("pixelwar.groups");
}

use proto::group_service_server::GroupService;
pub use proto::group_service_server::GroupServiceServer;
use proto::*;
use shared_common::errors::AppError;

pub struct GroupGrpcService {
    pub state: GroupState,
}

#[tonic::async_trait]
impl GroupService for GroupGrpcService {
    async fn create_group(
        &self,
        request: Request<CreateGroupRequest>,
    ) -> Result<Response<GroupReply>, Status> {
        let req = request.into_inner();
        let creator_id: uuid::Uuid = req
            .creator_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid creator_id"))?;
        let round_id: uuid::Uuid = req
            .round_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid round_id"))?;

        if req.name.is_empty() || req.name.len() > 100 {
            return Err(Status::invalid_argument(
                "Group name must be 1-100 characters",
            ));
        }

        let cmd = CreateGroupCommand {
            name: req.name,
            creator_id,
            round_id,
        };

        let output = self
            .state
            .create_group
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(GroupReply {
            group_id: output.group_id.to_string(),
            name: output.name,
            creator_id: output.creator_id.to_string(),
            member_ids: output.member_ids.iter().map(|id| id.to_string()).collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    async fn get_group(
        &self,
        request: Request<GetGroupRequest>,
    ) -> Result<Response<GroupReply>, Status> {
        let group_id: uuid::Uuid = request
            .into_inner()
            .group_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid group_id"))?;

        let output = self
            .state
            .get_group
            .execute(group_id)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(GroupReply {
            group_id: output.group_id.to_string(),
            name: output.name,
            creator_id: output.creator_id.to_string(),
            member_ids: output.member_ids.iter().map(|id| id.to_string()).collect(),
            created_at: output.created_at.to_rfc3339(),
        }))
    }

    async fn send_invite(
        &self,
        request: Request<SendInviteRequest>,
    ) -> Result<Response<InviteReply>, Status> {
        let req = request.into_inner();
        let group_id: uuid::Uuid = req
            .group_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid group_id"))?;
        let from_user: uuid::Uuid = req
            .from_user_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid from_user_id"))?;
        let to_user: uuid::Uuid = req
            .to_user_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid to_user_id"))?;

        let cmd = SendInviteCommand {
            group_id,
            from_user,
            to_user,
        };

        let output = self
            .state
            .send_invite
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(InviteReply {
            invite_id: output.invite_id.to_string(),
        }))
    }

    async fn accept_invite(
        &self,
        request: Request<AcceptInviteRequest>,
    ) -> Result<Response<GroupReply>, Status> {
        let req = request.into_inner();
        let invite_id: uuid::Uuid = req
            .invite_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid invite_id"))?;
        let user_id: uuid::Uuid = req
            .user_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let cmd = AcceptInviteCommand { invite_id, user_id };

        let _output = self
            .state
            .accept_invite
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        // Fetch the group to return full details
        let group_output = self
            .state
            .get_group
            .execute(_output.group_id)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(GroupReply {
            group_id: group_output.group_id.to_string(),
            name: group_output.name,
            creator_id: group_output.creator_id.to_string(),
            member_ids: group_output
                .member_ids
                .iter()
                .map(|id| id.to_string())
                .collect(),
            created_at: group_output.created_at.to_rfc3339(),
        }))
    }

    async fn get_user_group(
        &self,
        request: Request<GetUserGroupRequest>,
    ) -> Result<Response<GroupReply>, Status> {
        let req = request.into_inner();
        let user_id: uuid::Uuid = req
            .user_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;
        let round_id: uuid::Uuid = req
            .round_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid round_id"))?;

        let output = self
            .state
            .get_user_group
            .execute(user_id, round_id)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(GroupReply {
            group_id: output.group_id.to_string(),
            name: output.name,
            creator_id: output.creator_id.to_string(),
            member_ids: output.member_ids.iter().map(|id| id.to_string()).collect(),
            created_at: output.created_at.to_rfc3339(),
        }))
    }
}
