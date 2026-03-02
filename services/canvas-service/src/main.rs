mod adapters;
mod application;
mod domain;
mod grpc;
mod ports;
mod state;
mod storage;

use std::sync::Arc;

use tonic::transport::Server as TonicServer;
use tracing::info;

use adapters::event_publisher::NatsEventPublisher;
use adapters::pixel_cache::RedisPixelCache;
use adapters::repository::PgParcelRepository;
use application::{claim_parcel, get_parcel, update_pixels, get_snapshot, check_overlap};
use grpc::server::{CanvasGrpcService, CanvasServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("canvas-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9192)?;

    // ── Infrastructure ──────────────────────────────────────────────────────
    let db    = shared_db::postgres::create_pool(&config.database_url).await?;
    let redis = shared_db::redis::create_connection_manager(&config.redis_url).await?;
    let nats  = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    // ── Adapters ────────────────────────────────────────────────────────────
    let parcel_repo: Arc<dyn ports::ParcelRepository> = Arc::new(PgParcelRepository::new(db));
    let pixel_cache: Arc<dyn ports::PixelCache>       = Arc::new(RedisPixelCache::new(redis));
    let events: Arc<dyn ports::EventPublisher>        = Arc::new(NatsEventPublisher::new(nats));

    // ── Use cases ───────────────────────────────────────────────────────────
    let claim_parcel_uc = Arc::new(claim_parcel::ClaimParcel::new(
        Arc::clone(&parcel_repo),
        Arc::clone(&events),
    ));
    let get_parcel_uc = Arc::new(get_parcel::GetParcel::new(Arc::clone(&parcel_repo)));
    let update_pixels_uc = Arc::new(update_pixels::UpdatePixels::new(
        Arc::clone(&parcel_repo),
        Arc::clone(&pixel_cache),
        Arc::clone(&events),
    ));
    let get_snapshot_uc = Arc::new(get_snapshot::GetSnapshot::new(Arc::clone(&pixel_cache)));
    let check_overlap_uc = Arc::new(check_overlap::CheckOverlap::new(Arc::clone(&parcel_repo)));

    // ── Application state ───────────────────────────────────────────────────
    let state = state::CanvasState {
        claim_parcel: claim_parcel_uc,
        get_parcel: get_parcel_uc,
        update_pixels: update_pixels_uc,
        get_snapshot: get_snapshot_uc,
        check_overlap: check_overlap_uc,
    };

    // ── HTTP health server ──────────────────────────────────────────────────
    let http_app = axum::Router::new()
        .merge(health_routes(&config.service_name));

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Canvas service starting");

    // ── gRPC server ─────────────────────────────────────────────────────────
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
