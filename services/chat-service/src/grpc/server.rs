use tonic::{Request, Response, Status};
use uuid::Uuid;

use shared_common::errors::AppError;

use crate::domain::commands::{SendMessageCommand, GetMessagesQuery};
use crate::state::ChatState;

pub mod proto {
    tonic::include_proto!("pixelwar.chat");
}

use proto::chat_service_server::ChatService;
pub use proto::chat_service_server::ChatServiceServer;
use proto::*;

pub struct ChatGrpcService {
    pub state: ChatState,
}

#[tonic::async_trait]
impl ChatService for ChatGrpcService {
    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageReply>, Status> {
        let req = request.into_inner();

        // Parse sender_id
        let sender_id: Uuid = req
            .sender_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid sender_id"))?;

        // Parse channel_id if provided
        let channel_id: Option<Uuid> = if req.channel_id.is_empty() {
            None
        } else {
            Some(
                req.channel_id
                    .parse()
                    .map_err(|_| Status::invalid_argument("Invalid channel_id"))?,
            )
        };

        // Create command
        let cmd = SendMessageCommand {
            channel_type: req.channel_type,
            channel_id,
            sender_id,
            sender_name: req.sender_name,
            content: req.content,
        };

        // Execute use case
        let output = self
            .state
            .send_message
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(SendMessageReply {
            message_id: output.message_id.to_string(),
        }))
    }

    async fn get_messages(
        &self,
        request: Request<GetMessagesRequest>,
    ) -> Result<Response<GetMessagesReply>, Status> {
        let req = request.into_inner();

        // Parse channel_id if provided
        let channel_id: Option<Uuid> = if req.channel_id.is_empty() {
            None
        } else {
            Some(
                req.channel_id
                    .parse()
                    .map_err(|_| Status::invalid_argument("Invalid channel_id"))?,
            )
        };

        // Parse before_id cursor if provided
        let before_id: Option<Uuid> = if req.before_id.is_empty() {
            None
        } else {
            Some(
                req.before_id
                    .parse()
                    .map_err(|_| Status::invalid_argument("Invalid before_id"))?,
            )
        };

        // Create query
        let query = GetMessagesQuery {
            channel_type: req.channel_type,
            channel_id,
            limit: req.limit,
            before_id,
        };

        // Execute use case
        let output = self
            .state
            .get_messages
            .execute(query)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        // Map to proto messages
        let messages: Vec<ChatMessageProto> = output
            .messages
            .into_iter()
            .map(|msg| ChatMessageProto {
                id: msg.id.to_string(),
                sender_id: msg.sender_id.to_string(),
                sender_name: msg.sender_name,
                channel_type: msg.channel_type,
                channel_id: msg.channel_id.map(|id| id.to_string()).unwrap_or_default(),
                content: msg.content,
                created_at: msg.created_at.to_rfc3339(),
            })
            .collect();

        Ok(Response::new(GetMessagesReply { messages }))
    }
}
