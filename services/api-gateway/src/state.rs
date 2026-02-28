use redis::aio::ConnectionManager;
use shared_messaging::NatsClient;
use std::sync::Arc;


/// Shared application state for the API Gateway.
#[derive(Clone)]
pub struct AppState {
    pub redis: ConnectionManager,
    pub nats: NatsClient,
    pub jwt_secret: String,
}


impl AppState {
    pub fn new(redis: ConnectionManager, nats: NatsClient, jwt_secret: String) -> Self {
        Self { redis, nats, jwt_secret }
    }
}
