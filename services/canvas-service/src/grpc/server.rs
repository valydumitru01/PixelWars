use tonic::{Request, Response, Status};

use crate::domain::commands::{
    CheckOverlapQuery, ClaimParcelCommand, PixelUpdate, SnapshotQuery, UpdatePixelsCommand,
};
use crate::state::CanvasState;
use shared_common::errors::AppError;

pub mod proto {
    tonic::include_proto!("pixelwar.canvas");
}

use proto::canvas_service_server::CanvasService;
pub use proto::canvas_service_server::CanvasServiceServer;
use proto::*;

pub struct CanvasGrpcService {
    pub state: CanvasState,
}

#[tonic::async_trait]
impl CanvasService for CanvasGrpcService {
    async fn claim_parcel(
        &self,
        request: Request<ClaimParcelRequest>,
    ) -> Result<Response<ParcelReply>, Status> {
        let req = request.into_inner();

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;
        let round_id = uuid::Uuid::parse_str(&req.round_id)
            .map_err(|_| Status::invalid_argument("Invalid round_id"))?;

        let cmd = ClaimParcelCommand {
            user_id,
            round_id,
            origin_x: req.origin_x,
            origin_y: req.origin_y,
            width: req.width,
            height: req.height,
            description: req.description,
        };

        let output = self
            .state
            .claim_parcel
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(ParcelReply {
            parcel_id: output.parcel_id.to_string(),
            user_id: output.user_id.to_string(),
            round_id: output.round_id.to_string(),
            origin_x: output.origin_x,
            origin_y: output.origin_y,
            width: output.width,
            height: output.height,
            description: output.description,
            created_at: output.created_at.to_rfc3339(),
        }))
    }

    async fn get_parcel(
        &self,
        request: Request<GetParcelRequest>,
    ) -> Result<Response<ParcelReply>, Status> {
        let parcel_id = uuid::Uuid::parse_str(&request.into_inner().parcel_id)
            .map_err(|_| Status::invalid_argument("Invalid parcel_id"))?;

        let output = self
            .state
            .get_parcel
            .execute(parcel_id)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(ParcelReply {
            parcel_id: output.parcel_id.to_string(),
            user_id: output.user_id.to_string(),
            round_id: output.round_id.to_string(),
            origin_x: output.origin_x,
            origin_y: output.origin_y,
            width: output.width,
            height: output.height,
            description: output.description,
            created_at: output.created_at.to_rfc3339(),
        }))
    }

    async fn update_pixels(
        &self,
        request: Request<UpdatePixelsRequest>,
    ) -> Result<Response<UpdatePixelsReply>, Status> {
        let req = request.into_inner();

        let parcel_id = uuid::Uuid::parse_str(&req.parcel_id)
            .map_err(|_| Status::invalid_argument("Invalid parcel_id"))?;
        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let pixels = req
            .pixels
            .into_iter()
            .map(|px| PixelUpdate {
                local_x: px.local_x,
                local_y: px.local_y,
                color: px.color,
            })
            .collect();

        let cmd = UpdatePixelsCommand {
            parcel_id,
            user_id,
            pixels,
        };

        let output = self
            .state
            .update_pixels
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(UpdatePixelsReply {
            updated_count: output.updated_count,
        }))
    }

    async fn get_snapshot(
        &self,
        request: Request<GetSnapshotRequest>,
    ) -> Result<Response<SnapshotReply>, Status> {
        let req = request.into_inner();

        let query = SnapshotQuery {
            x: req.x,
            y: req.y,
            width: req.width,
            height: req.height,
        };

        let output = self
            .state
            .get_snapshot
            .execute(query)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(SnapshotReply {
            data: output.data,
            width: output.width,
            height: output.height,
        }))
    }

    async fn check_overlap(
        &self,
        request: Request<CheckOverlapRequest>,
    ) -> Result<Response<CheckOverlapReply>, Status> {
        let req = request.into_inner();

        let round_id = uuid::Uuid::parse_str(&req.round_id)
            .map_err(|_| Status::invalid_argument("Invalid round_id"))?;

        let query = CheckOverlapQuery {
            round_id,
            origin_x: req.origin_x,
            origin_y: req.origin_y,
            width: req.width,
            height: req.height,
        };

        let output = self
            .state
            .check_overlap
            .execute(query)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(CheckOverlapReply {
            overlaps: output.overlaps,
        }))
    }
}
