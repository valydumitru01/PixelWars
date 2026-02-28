use redis::aio::ConnectionManager;
use shared_messaging::NatsClient;

#[derive(Clone)]
pub struct ChatState {
    pub redis: ConnectionManager,
    pub nats: NatsClient,
}
