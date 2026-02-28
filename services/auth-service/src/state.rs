use shared_messaging::NatsClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AuthState {
    pub db: PgPool,
    pub nats: NatsClient,
    pub jwt_secret: String,
}
