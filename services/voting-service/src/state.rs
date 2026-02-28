use shared_messaging::NatsClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct VotingState {
    pub db: PgPool,
    pub nats: NatsClient,
}
