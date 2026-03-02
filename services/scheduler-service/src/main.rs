mod adapters;
mod application;
mod domain;
mod jobs;
mod ports;
mod state;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use tokio::time;
use tracing::{info, error};

use adapters::event_publisher::NatsEventPublisher;
use adapters::repository::{PgParcelRepository, PgRoundRepository, PgUserRepository};
use application::{close_voting, end_round, run_activity_check, start_round};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("scheduler-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9196)?;

    // ── Infrastructure ──────────────────────────────────────────────────────
    let db   = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    // ── Adapters ────────────────────────────────────────────────────────────
    let user_repo: Arc<dyn ports::UserRepository> = Arc::new(PgUserRepository::new(db.clone()));
    let parcel_repo: Arc<dyn ports::ParcelRepository> = Arc::new(PgParcelRepository::new(db.clone()));
    let round_repo: Arc<dyn ports::RoundRepository> = Arc::new(PgRoundRepository::new(db.clone()));
    let events: Arc<dyn ports::EventPublisher> = Arc::new(NatsEventPublisher::new(nats));

    // ── Use cases ───────────────────────────────────────────────────────────
    let activity_check = Arc::new(run_activity_check::RunActivityCheck::new(
        Arc::clone(&user_repo),
        Arc::clone(&parcel_repo),
        Arc::clone(&events),
    ));
    let end_round_uc = Arc::new(end_round::EndRound::new(
        Arc::clone(&round_repo),
        Arc::clone(&events),
    ));
    let close_voting_uc = Arc::new(close_voting::CloseVoting::new(
        Arc::clone(&round_repo),
        Arc::clone(&events),
    ));
    let start_round_uc = Arc::new(start_round::StartRound::new(
        Arc::clone(&round_repo),
        Arc::clone(&events),
    ));

    // ── Application state ───────────────────────────────────────────────────
    let state = state::SchedulerState {
        run_activity_check: activity_check,
        end_round: end_round_uc,
        close_voting: close_voting_uc,
        start_round: start_round_uc,
        round_repo: Arc::clone(&round_repo),
    };

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
            if let Err(e) = jobs::activity_check::run_activity_check_job(&activity_state).await {
                error!(error = %e, "Activity check failed");
            }
        }
    });

    // Round end check: every hour
    let round_state = state.clone();
    let round_job = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;

            // Check if active round has expired and end it
            if let Err(e) = jobs::round_lifecycle::end_expired_round_job(&round_state).await {
                error!(error = %e, "Failed to end round");
            }

            // Close expired voting windows
            if let Err(e) = jobs::voting_window::close_expired_voting_job(&round_state).await {
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
