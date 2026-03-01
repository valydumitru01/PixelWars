mod grpc;
mod handlers;
mod rooms;
mod state;

use axum::{routing::post, Router};
use tonic::transport::Server as TonicServer;
use tracing::info;

use grpc::server::{ChatGrpcService, ChatServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("chat-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9193)?;

    let db    = shared_db::postgres::create_pool(&config.database_url).await?;
    let redis = shared_db::redis::create_connection_manager(&config.redis_url).await?;
    let nats  = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::ChatState { db, redis, nats };

    let http_app = Router::new()
        .merge(health_routes(&config.service_name))
        .route("/global",  post(handlers::global::send_global_message))
        .route("/group",   post(handlers::group::send_group_message))
        .route("/whisper", post(handlers::whisper::send_whisper))
        .with_state(state.clone());

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Chat service starting");

    let grpc_svc = ChatGrpcService { state };
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
