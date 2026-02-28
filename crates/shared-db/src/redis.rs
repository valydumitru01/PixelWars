use redis::aio::ConnectionManager;
use tracing::info;

/// Create a Redis connection manager for async use.
pub async fn create_connection_manager(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    let manager = ConnectionManager::new(client).await?;
    info!("Redis connection manager established");
    Ok(manager)
}
