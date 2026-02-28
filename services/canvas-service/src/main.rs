mod domain;
mod handlers;
mod state;
mod storage;

use axum::{routing::{get, post}, Router};
use tracing::info;

use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("canvas-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9092)?;

    let db = shared_db::postgres::create_pool(&config.database_url).await?;
    let redis = shared_db::redis::create_connection_manager(&config.redis_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::CanvasState { db, redis, nats };

    let app = Router::new()
        .route("/parcels", post(handlers::parcel::claim_parcel))
        .route("/pixels", post(handlers::pixel::update_pixels))
        .route("/snapshot", get(handlers::snapshot::get_snapshot))
        .with_state(state)
        .merge(health_routes(&config.service_name));

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "Canvas service starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
