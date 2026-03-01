mod domain;
mod grpc;
mod handlers;
mod state;
mod storage;

use axum::{routing::{get, post}, Router};
use tonic::transport::Server as TonicServer;
use tracing::info;

use grpc::server::{CanvasGrpcService, CanvasServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("canvas-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9192)?;

    let db    = shared_db::postgres::create_pool(&config.database_url).await?;
    let redis = shared_db::redis::create_connection_manager(&config.redis_url).await?;
    let nats  = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::CanvasState { db, redis, nats };

    let http_app = Router::new()
        .route("/parcels", post(handlers::parcel::claim_parcel))
        .route("/pixels",  post(handlers::pixel::update_pixels))
        .route("/snapshot", get(handlers::snapshot::get_snapshot))
        .with_state(state.clone())
        .merge(health_routes(&config.service_name));

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Canvas service starting");

    let grpc_svc = CanvasGrpcService { state };
    let grpc_server = TonicServer::builder()
        .add_service(CanvasServiceServer::new(grpc_svc))
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
