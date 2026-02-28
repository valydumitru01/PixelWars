mod handlers;
mod state;
mod tallying;

use axum::{routing::{get, post}, Router};
use tracing::info;

use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("voting-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9094)?;

    let db = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::VotingState { db, nats };

    let app = Router::new()
        .route("/vote", post(handlers::cast_vote::cast_vote))
        .route("/results", get(handlers::results::get_results))
        .with_state(state)
        .merge(health_routes(&config.service_name));

    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "Voting service starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
