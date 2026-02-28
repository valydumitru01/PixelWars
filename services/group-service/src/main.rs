mod handlers;
mod state;

use axum::{routing::post, Router};
use tracing::info;

use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("group-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9095)?;

    let db = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::GroupState { db, nats };

    let app = Router::new()
        .route("/", post(handlers::create::create_group))
        .route("/invite", post(handlers::invite::invite_member))
        .route("/invite/accept", post(handlers::invite::accept_invite))
        .with_state(state)
        .merge(health_routes(&config.service_name));

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "Group service starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
