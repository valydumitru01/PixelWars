mod middleware;
mod routes;
mod state;
mod websocket;

use axum::{routing::get, Router};
use axum::handler::Handler;
use axum::middleware::from_fn;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};

use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};
use crate::middleware::auth::require_auth;
use crate::middleware::rate_limit::rate_limit;
use crate::middleware::tracing::request_tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config: ServiceConfig = ServiceConfig::from_env("api-gateway")?;
    init_tracing(config.service_name.as_str(), config.otel_endpoint.as_str())?;
    init_metrics(9090)?;

    let redis = shared_db::redis::create_connection_manager(&config.redis_url.as_str()).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url.as_str()).await?;
    let app_state = state::AppState::new(redis, nats, config.jwt_secret.clone());
    let _span = info_span!("gateway_startup_check").entered();
    tracing::info!("Tracing is active!");
    let app = Router::new()
        .merge(routes::api_routes(app_state.clone()))
        .route(
            "/ws",
            get(websocket::handler::ws_handler).with_state(app_state),
        )
        .merge(health_routes(&config.service_name))
        .layer(from_fn(rate_limit))
        .layer(from_fn(require_auth))
        .layer(from_fn(request_tracing))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "API Gateway starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
