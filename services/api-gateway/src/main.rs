mod clients;
mod middleware;
mod routes;
mod state;
mod websocket;

use axum::error_handling::HandleErrorLayer;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::middleware::from_fn;
use axum::{routing::get, Extension, Router};
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

use clients::auth::auth_service_client::AuthServiceClient;
use clients::canvas::canvas_service_client::CanvasServiceClient;
use clients::chat::chat_service_client::ChatServiceClient;
use clients::voting::voting_service_client::VotingServiceClient;
use clients::groups::group_service_client::GroupServiceClient;
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

use crate::middleware::auth::require_auth;
use crate::middleware::tracing::request_tracing;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("api-gateway")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9090)?;

    let redis = shared_db::redis::create_connection_manager(&config.redis_url).await?;
    let nats  = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    // Connect gRPC clients to downstream services
    info!("Connecting to downstream gRPC services...");
    let auth_client   = AuthServiceClient::connect(config.auth_grpc_url.clone()).await?;
    let canvas_client = CanvasServiceClient::connect(config.canvas_grpc_url.clone()).await?;
    let chat_client   = ChatServiceClient::connect(config.chat_grpc_url.clone()).await?;
    let voting_client = VotingServiceClient::connect(config.voting_grpc_url.clone()).await?;
    let group_client  = GroupServiceClient::connect(config.group_grpc_url.clone()).await?;
    info!("All gRPC clients connected");

    let app_state = state::AppState {
        redis,
        nats,
        jwt_secret: config.jwt_secret.clone(),
        auth_client,
        canvas_client,
        chat_client,
        voting_client,
        group_client,
    };

    let _span = info_span!("gateway_startup_check").entered();

    let rate_limit_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            if err.is::<Elapsed>() {
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Unhandled error: {}", err))
            }
        }))
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
        ));

    // Inject jwt_secret into all requests so auth middleware can access it
    let jwt_secret = config.jwt_secret.clone();

    let public_routes = Router::new()
        .merge(routes::public_routes())
        .merge(health_routes::<AppState>(&config.service_name));

    let private_routes = Router::new()
        .merge(routes::private_routes())
        .layer(from_fn(require_auth))
        .layer(Extension(jwt_secret));

    let app = Router::new()
        .merge(public_routes)
        .merge(private_routes)
        .layer(from_fn(request_tracing))
        .layer(TraceLayer::new_for_http())
        .layer(rate_limit_layer)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(timeout_layer)
        .layer(http_header_layers)
        .layer(PropagateHeaderLayer::new(HeaderName::from_static("x-request-id")))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "API Gateway starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
