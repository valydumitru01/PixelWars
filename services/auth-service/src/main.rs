mod adapters;
mod application;
mod domain;
mod grpc;
mod ports;
mod state;

use std::sync::Arc;

use tonic::transport::Server as TonicServer;
use tracing::info;

use adapters::event_publisher::NatsEventPublisher;
use adapters::password_hasher::Argon2PasswordHasher;
use adapters::repository::PgUserRepository;
use adapters::token_provider::JwtTokenProvider;
use application::{get_user, login_user, register_user, validate_token};
use grpc::server::{AuthGrpcService, AuthServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("auth-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9191)?;

    // ── Infrastructure ──────────────────────────────────────────────────────
    let db   = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    // ── Adapters ────────────────────────────────────────────────────────────
    let user_repo: Arc<dyn ports::UserRepository> = Arc::new(PgUserRepository::new(db));
    let hasher: Arc<dyn ports::PasswordHasher>     = Arc::new(Argon2PasswordHasher);
    let tokens: Arc<dyn ports::TokenProvider>      = Arc::new(JwtTokenProvider::new(config.jwt_secret.clone()));
    let events: Arc<dyn ports::EventPublisher>     = Arc::new(NatsEventPublisher::new(nats));

    // ── Use cases ───────────────────────────────────────────────────────────
    let register = Arc::new(register_user::RegisterUser::new(
        Arc::clone(&user_repo),
        Arc::clone(&hasher),
        Arc::clone(&tokens),
        Arc::clone(&events),
    ));
    let login = Arc::new(login_user::LoginUser::new(
        Arc::clone(&user_repo),
        Arc::clone(&hasher),
        Arc::clone(&tokens),
        Arc::clone(&events),
    ));
    let validate = Arc::new(validate_token::ValidateToken::new(Arc::clone(&tokens)));
    let get_user_uc = Arc::new(get_user::GetUser::new(Arc::clone(&user_repo)));

    // ── Application state ───────────────────────────────────────────────────
    let state = state::AuthState {
        register_user: register,
        login_user: login,
        validate_token: validate,
        get_user: get_user_uc,
    };

    // ── HTTP health server ──────────────────────────────────────────────────
    let http_app = axum::Router::new()
        .merge(health_routes(&config.service_name));

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Auth service starting");

    // ── gRPC server ─────────────────────────────────────────────────────────
    let grpc_svc = AuthGrpcService { state };
    let grpc_server = TonicServer::builder()
        .add_service(AuthServiceServer::new(grpc_svc))
        .serve(grpc_addr);

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
