use shared_messaging::NatsClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct SchedulerState {
    pub db: PgPool,
    pub nats: NatsClient,
}
