mod handlers;
mod rooms;
mod state;

use axum::{routing::post, Router};
use tracing::info;

use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("chat-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9093)?;

    let redis = shared_db::redis::create_connection_manager(&config.redis_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::ChatState { redis, nats };

    let app = Router::new()
        .route("/global", post(handlers::global::send_global_message))
        .route("/group", post(handlers::group::send_group_message))
        .route("/whisper", post(handlers::whisper::send_whisper))
        .with_state(state)
        .merge(health_routes(&config.service_name));

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "Chat service starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
