mod jobs;
mod state;

use std::time::Duration;
use tokio::time;
use tracing::info;

use shared_common::config::ServiceConfig;
use shared_observability::{init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("scheduler-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9096)?;

    let db = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::SchedulerState { db, nats };

    info!("Scheduler service starting — running periodic jobs");

    // Activity check: runs every 6 hours
    let activity_state = state.clone();
    let activity_handle = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(6 * 3600));
        loop {
            interval.tick().await;
            // TODO: Get current round_id from DB
            let round_id = uuid::Uuid::new_v4(); // placeholder
            if let Err(e) = jobs::activity_check::run_activity_check(&activity_state, round_id).await {
                tracing::error!(error = %e, "Activity check failed");
            }
        }
    });

    // Keep the service running
    activity_handle.await?;

    Ok(())
}
