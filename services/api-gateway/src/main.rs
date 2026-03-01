mod middleware;
mod routes;
mod state;
mod websocket;

use axum::error_handling::HandleErrorLayer;
use axum::handler::Handler;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::middleware::from_fn;
use axum::{routing::get, Router};
use axum::routing::post;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::{BoxError, ServiceBuilder};
use tower::timeout::error::Elapsed;
use tower::timeout::TimeoutLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::propagate_header::PropagateHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};

use crate::middleware::auth::require_auth;
use crate::middleware::tracing::request_tracing;
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

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
    info!("Tracing is active!");
    // Rate limiter can't be added right away because it is not clonable and axum clones the router
    // for each request for concurrent handling.
    // This buffer layer changes the architecture of the middleware to allow for the rate limiter to
    // be shared across all requests without cloning it.
    let rate_limit_layer = ServiceBuilder::new()
        // HandleErrorLayer is used to catch errors from the rate limiter and return a proper
        // response instead of crashing the server.
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            if err.is::<Elapsed>() {
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Unhandled error: {}", err))
            }
        }))
        // The buffer has 1024 slots for pending requests. If the rate limiter is currently
        // processing the maximum allowed requests (5 in this case), additional requests will be
        // buffered up to this limit instead of being rejected immediately.
        // If the buffer is full, new requests will be rejected and an error will be returned.
        .layer(BufferLayer::new(1024))
        .layer(RateLimitLayer::new(5, std::time::Duration::from_secs(1)));

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_| async {
            (StatusCode::REQUEST_TIMEOUT, "Request timed out")
        }))
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)));

    let http_header_layers = ServiceBuilder::new()
        .layer(tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("content-security-policy"),
                HeaderValue::from_static("default-src 'self'"),
        ));

    // 1. Routes that DON'T need a token
    let public_routes = Router::new()
        .merge(routes::public_routes(app_state.clone()))
        .merge(health_routes(&config.service_name));

    // 2. Routes that DO need a token
    let private_routes = Router::new()
        .merge(routes::private_routes(app_state.clone()))
        // -- Inner Layers (Business) --
        .layer(from_fn(require_auth));

    let app = Router::new()
        .merge(public_routes)
        .merge(private_routes)
        // -- Middle Layers (Observability) --
        .layer(from_fn(request_tracing))
        .layer(TraceLayer::new_for_http())
        // -- Outer Layers (Protection) --
        .layer(rate_limit_layer)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(timeout_layer)
        .layer(http_header_layers)
        .layer(PropagateHeaderLayer::new(HeaderName::from_static("x-request-id")))
        .layer(CorsLayer::permissive());

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "API Gateway starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
