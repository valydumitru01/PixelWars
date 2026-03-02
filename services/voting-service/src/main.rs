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
use adapters::repository::{PgRoundRepository, PgVoteRepository};
use application::{cast_vote, get_results, is_voting_open};
use grpc::server::{VotingGrpcService, VotingServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("voting-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9194)?;

    // ── Infrastructure ──────────────────────────────────────────────────────
    let db = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    // ── Adapters ────────────────────────────────────────────────────────────
    let vote_repo: Arc<dyn ports::VoteRepository> = Arc::new(PgVoteRepository::new(db.clone()));
    let round_repo: Arc<dyn ports::RoundRepository> = Arc::new(PgRoundRepository::new(db));
    let events: Arc<dyn ports::EventPublisher> = Arc::new(NatsEventPublisher::new(nats));

    // ── Use cases ───────────────────────────────────────────────────────────
    let cast_vote_uc = Arc::new(cast_vote::CastVote::new(
        Arc::clone(&vote_repo),
        Arc::clone(&round_repo),
        Arc::clone(&events),
    ));
    let get_results_uc = Arc::new(get_results::GetResults::new(Arc::clone(&vote_repo)));
    let is_voting_open_uc = Arc::new(is_voting_open::IsVotingOpen::new(Arc::clone(&round_repo)));

    // ── Application state ───────────────────────────────────────────────────
    let state = state::VotingState {
        cast_vote: cast_vote_uc,
        get_results: get_results_uc,
        is_voting_open: is_voting_open_uc,
    };

    // ── HTTP health server ──────────────────────────────────────────────────
    let http_app = axum::Router::new()
        .merge(health_routes(&config.service_name));

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Voting service starting");

    // ── gRPC server ─────────────────────────────────────────────────────────
    let grpc_svc = VotingGrpcService { state };
    let grpc_server = TonicServer::builder()
        .add_service(VotingServiceServer::new(grpc_svc))
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
