mod grpc;
mod handlers;
mod jwt;
mod password;
mod state;

use axum::{routing::{get, post}, Router};
use tonic::transport::Server as TonicServer;
use tracing::info;

use grpc::server::{AuthGrpcService, AuthServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};
use state::AuthState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("auth-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9191)?;

    let db   = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = AuthState { db: db.clone(), nats, jwt_secret: config.jwt_secret.clone() };

    // ── HTTP server ──────────────────────────────────────────────────────────
    let http_app = Router::new()
        .merge(health_routes(&config.service_name))
        .route("/register", post(handlers::register::handle_register))
        .route("/login",    post(handlers::login::handle_login))
        .route("/session",  get(handlers::session::validate_session))
        .with_state(state.clone());

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Auth service starting");

    // ── gRPC server ───────────────────────────────────────────────────────────
    let grpc_svc = AuthGrpcService { state };
    let grpc_server = TonicServer::builder()
        .add_service(AuthServiceServer::new(grpc_svc))
        .serve(grpc_addr);

    // Run both concurrently
    tokio::try_join!(
        async {
            let listener = tokio::net::TcpListener::bind(http_addr).await?;
            axum::serve(listener, http_app).await?;
            Ok::<_, anyhow::Error>(())
        },
        async {
            grpc_server.await.map_err(anyhow::Error::from)
        }
    )?;

    Ok(())
}
