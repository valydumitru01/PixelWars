mod grpc;
mod handlers;
mod state;

use axum::{routing::post, Router};
use tonic::transport::Server as TonicServer;
use tracing::info;

use grpc::server::{GroupGrpcService, GroupServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("group-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9195)?;

    let db   = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::GroupState { db, nats };

    let http_app = Router::new()
        .merge(health_routes(&config.service_name))
        .route("/",              post(handlers::create::create_group))
        .route("/invite",        post(handlers::invite::invite_member))
        .route("/invite/accept", post(handlers::invite::accept_invite))
        .with_state(state.clone());

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Group service starting");

    let grpc_svc = GroupGrpcService { state };
    let grpc_server = TonicServer::builder()
        .add_service(GroupServiceServer::new(grpc_svc))
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
