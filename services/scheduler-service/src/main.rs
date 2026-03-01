mod jobs;
mod state;

use std::time::Duration;
use axum::Router;
use tokio::time;
use tracing::{info, error};

use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("scheduler-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9196)?;

    let db   = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    let state = state::SchedulerState { db, nats };

    info!("Scheduler service starting");

    // Health check HTTP server
    let http_app = Router::new().merge(health_routes(&config.service_name));
    let http_addr = config.http_addr();

    // Activity check: every 6 hours
    let activity_state = state.clone();
    let activity_job = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(6 * 3600));
        loop {
            interval.tick().await;
            match jobs::round_lifecycle::get_active_round(&activity_state).await {
                Ok(Some(round_id)) => {
                    if let Err(e) = jobs::activity_check::run_activity_check(&activity_state, round_id).await {
                        error!(error = %e, "Activity check failed");
                    }
                }
                Ok(None) => info!("No active round, skipping activity check"),
                Err(e) => error!(error = %e, "Failed to get active round"),
            }
        }
    });

    // Round end check: every hour
    let round_state = state.clone();
    let round_job = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            // Check if active round has expired
            let maybe_expired = sqlx::query!(
                "SELECT id FROM rounds WHERE is_active = true AND ends_at < NOW() LIMIT 1"
            )
            .fetch_optional(&round_state.db)
            .await;

            if let Ok(Some(round)) = maybe_expired {
                info!(round_id = %round.id, "Round expired, ending it...");
                if let Err(e) = jobs::round_lifecycle::end_round(&round_state, round.id).await {
                    error!(error = %e, "Failed to end round");
                }
            }

            // Close expired voting windows
            if let Err(e) = jobs::voting_window::close_expired_voting(&round_state).await {
                error!(error = %e, "Failed to close voting windows");
            }
        }
    });

    // Run HTTP server alongside jobs
    tokio::try_join!(
        async {
            let listener = tokio::net::TcpListener::bind(http_addr).await?;
            info!(addr = %http_addr, "Scheduler HTTP health server starting");
            axum::serve(listener, http_app).await?;
            Ok::<_, anyhow::Error>(())
        },
        async { activity_job.await.map_err(|e| anyhow::anyhow!(e)) },
        async { round_job.await.map_err(|e| anyhow::anyhow!(e)) },
    )?;

    Ok(())
}
