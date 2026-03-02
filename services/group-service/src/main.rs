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
use adapters::repository::{PgGroupRepository, PgInviteRepository};
use application::{accept_invite, create_group, get_group, get_user_group, send_invite};
use grpc::server::{GroupGrpcService, GroupServiceServer};
use shared_common::config::ServiceConfig;
use shared_observability::{health_routes, init_metrics, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServiceConfig::from_env("group-service")?;
    init_tracing(&config.service_name, &config.otel_endpoint)?;
    init_metrics(9195)?;

    // ── Infrastructure ──────────────────────────────────────────────────────
    let db = shared_db::postgres::create_pool(&config.database_url).await?;
    let nats = shared_messaging::NatsClient::connect(&config.nats_url).await?;

    // ── Adapters ────────────────────────────────────────────────────────────
    let group_repo: Arc<dyn ports::GroupRepository> = Arc::new(PgGroupRepository::new(db.clone()));
    let invite_repo: Arc<dyn ports::InviteRepository> = Arc::new(PgInviteRepository::new(db));
    let events: Arc<dyn ports::EventPublisher> = Arc::new(NatsEventPublisher::new(nats));

    // ── Use cases ───────────────────────────────────────────────────────────
    let create = Arc::new(create_group::CreateGroup::new(
        Arc::clone(&group_repo),
        Arc::clone(&events),
    ));
    let get = Arc::new(get_group::GetGroup::new(Arc::clone(&group_repo)));
    let send = Arc::new(send_invite::SendInvite::new(
        Arc::clone(&group_repo),
        Arc::clone(&invite_repo),
        Arc::clone(&events),
    ));
    let accept = Arc::new(accept_invite::AcceptInvite::new(
        Arc::clone(&group_repo),
        Arc::clone(&invite_repo),
        Arc::clone(&events),
    ));
    let get_user = Arc::new(get_user_group::GetUserGroup::new(Arc::clone(&group_repo)));

    // ── Application state ───────────────────────────────────────────────────
    let state = state::GroupState {
        create_group: create,
        get_group: get,
        send_invite: send,
        accept_invite: accept,
        get_user_group: get_user,
    };

    // ── HTTP health server ──────────────────────────────────────────────────
    let http_app = axum::Router::new()
        .merge(health_routes(&config.service_name));

    let http_addr = config.http_addr();
    let grpc_addr = config.grpc_addr();

    info!(http = %http_addr, grpc = %grpc_addr, "Group service starting");

    // ── gRPC server ─────────────────────────────────────────────────────────
    let grpc_svc = GroupGrpcService { state };
    let grpc_server = TonicServer::builder()
        .add_service(GroupServiceServer::new(grpc_svc))
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
