mod handlers;
mod jwt;
mod password;
mod state;

use axum::{routing::post, Router};
use tracing::info;

use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("auth-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9091)?;

    let db = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::AuthState {
        db,
        nats,
        jwt_secret: config.jwt_secret.clone(),
    };

    let app = Router::new()
        .route("/register", post(handlers::register::handle_register))
        .route("/login", post(handlers::login::handle_login))
        .with_state(state)
        .merge(health_routes(&config.service_name));

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "Auth service starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
