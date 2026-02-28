mod middleware;
mod routes;
mod state;
mod websocket;

use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;
use tracing::info;

use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("api-gateway")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9090)?;

    let redis = shared_db::redis::create_connection_manager(&config.redis_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;
    let app_state = state::AppState::new(redis, nats, config.jwt_secret.clone());

    let app = Router::new()
        .merge(health_routes(&config.service_name))
        .merge(routes::api_routes(app_state.clone()))
        .route("/ws", get(websocket::handler::ws_handler).with_state(app_state))
        .layer(CorsLayer::permissive());

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "API Gateway starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
