use redis::aio::ConnectionManager;
use shared_messaging::NatsClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ChatState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub nats: NatsClient,
}
