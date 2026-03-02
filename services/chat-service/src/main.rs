mod adapters;
mod application;
mod domain;
mod grpc;
mod ports;
mod state;

use std::sync::Arc;

use axum::Router;
use tonic::transport::Server as TonicServer;
use tracing::info;

use adapters::repository::PgMessageRepository;
use adapters::event_publisher::NatsEventPublisher;
use application::{send_message, get_messages};
use grpc::server::{ChatGrpcService, ChatServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("chat-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9193)?;

    // Initialize infrastructure adapters
    let db = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    // Create port implementations
    let message_repo: Arc<dyn ports::MessageRepository> =
        Arc::new(PgMessageRepository::new(db.clone()));
    let event_publisher: Arc<dyn ports::EventPublisher> =
        Arc::new(NatsEventPublisher::new(nats));

    // Create use cases
    let send_message_uc = Arc::new(send_message::SendMessage::new(
        message_repo.clone(),
        event_publisher,
    ));
    let get_messages_uc =
        Arc::new(get_messages::GetMessages::new(message_repo));

    // Create application state
    let app_state = state::ChatState {
        send_message: send_message_uc,
        get_messages: get_messages_uc,
    };

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Chat service starting");

    // HTTP router with health checks only
    let http_app = Router::new().merge(health_routes(&config.service_name));

    // gRPC service
    let grpc_svc = ChatGrpcService {
        state: app_state,
    };
    let grpc_server = TonicServer::builder()
        .add_service(ChatServiceServer::new(grpc_svc))
        .serve(grpc_addr);

    tokio::try_join!(
        async {
            let listener = tokio::net::TcpListener::bind(http_addr).await?;
            axum::serve(listener, http_app).await?;
            Ok::<_, anyhow::Error>(())
        },
        async { grpc_server.await.map_err(anyhow::Error::from) }
    )?;

    Ok(())
}
