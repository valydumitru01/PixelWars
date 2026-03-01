use crate::clients;
use redis::aio::ConnectionManager;
use shared_messaging::NatsClient;

use clients::auth::auth_service_client::AuthServiceClient;
use clients::canvas::canvas_service_client::CanvasServiceClient;
use clients::chat::chat_service_client::ChatServiceClient;
use clients::voting::voting_service_client::VotingServiceClient;
use clients::groups::group_service_client::GroupServiceClient;
use tonic::transport::Channel;

/// Shared application state for the API Gateway.
/// Holds gRPC client connections to all downstream services.
#[derive(Clone)]
pub struct AppState {
    pub redis: ConnectionManager,
    pub nats: NatsClient,
    pub jwt_secret: String,
    pub auth_client: AuthServiceClient<Channel>,
    pub canvas_client: CanvasServiceClient<Channel>,
    pub chat_client: ChatServiceClient<Channel>,
    pub voting_client: VotingServiceClient<Channel>,
    pub group_client: GroupServiceClient<Channel>,
}
