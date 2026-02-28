use shared_messaging::NatsClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct GroupState {
    pub db: PgPool,
    pub nats: NatsClient,
}
