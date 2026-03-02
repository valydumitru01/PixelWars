mod clients;
mod middleware;
mod routes;
mod state;
mod websocket;

use axum::error_handling::HandleErrorLayer;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::middleware::from_fn;
use axum::{routing::get, Extension, Router};
use tonic::transport::Endpoint;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::{BoxError, ServiceBuilder};
use tower::timeout::error::Elapsed;
use tower::timeout::TimeoutLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::propagate_header::PropagateHeaderLayer;
use tower_http::trace::TraceLayer;

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
    // Offset the HTTP port by 10000 to guarantee a unique metrics port per service.
    let metrics_port = config.port + 10000;
    init_metrics(metrics_port)?;

    let redis = shared_db::redis::create_connection_manager(&config.redis_url).await?;
    let nats  = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    // Connect gRPC clients to downstream services
    tracing::info!("Configuring downstream gRPC services (lazy connect)...");

    let auth_client = AuthServiceClient::new(
        Endpoint::from_shared(config.auth_grpc_url.clone())?.connect_lazy()
    );
    let canvas_client = CanvasServiceClient::new(
        Endpoint::from_shared(config.canvas_grpc_url.clone())?.connect_lazy()
    );
    let chat_client = ChatServiceClient::new(
        Endpoint::from_shared(config.chat_grpc_url.clone())?.connect_lazy()
    );
    let voting_client = VotingServiceClient::new(
        Endpoint::from_shared(config.voting_grpc_url.clone())?.connect_lazy()
    );
    let group_client = GroupServiceClient::new(
        Endpoint::from_shared(config.group_grpc_url.clone())?.connect_lazy()
    );

    tracing::info!("All gRPC channels configured");

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

    let _span = tracing::info_span!("gateway_startup_check").entered();

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

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            // This creates the span with standard OTel attributes
            tracing::info_span!(
                    "http_request",
                    http.method = %request.method(),
                    http.uri = %request.uri(),
                    user_id = tracing::field::Empty,
                    http.status_code = tracing::field::Empty, // We leave this empty to fill later
                    otel.name = format!("{} {}", request.method(), request.uri().path()),
                    otel.kind = "server",
                )
        })
        .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, span: &tracing::Span| {
            let status = response.status().as_u16();

            // Record the status code into the span's field
            span.record("http.status_code", status);

            // If it's a 5xx error, mark the span as an error in Jaeger
            if status >= 500 {
                span.record("otel.status_code", "ERROR");
            }

            tracing::info!(
                    latency = ?latency,
                    status = status,
                    "Finished processing request"
                );
        });
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
        .layer(rate_limit_layer)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(timeout_layer)
        .layer(http_header_layers)
        .layer(trace_layer)
        .layer(PropagateHeaderLayer::new(HeaderName::from_static("x-request-id")))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!(addr = %addr, "API Gateway starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
