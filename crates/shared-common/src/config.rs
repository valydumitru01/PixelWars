use serde::Deserialize;
/// Base configuration shared across all services.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub nats_url: String,
    pub jwt_secret: String,
    pub otel_endpoint: String,
}

impl ServiceConfig {
    /// Load config from environment variables.
    pub fn from_env(service_name: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            service_name: service_name.to_string(),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .as_str()
                .parse::<u16>()?,
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")?,
            otel_endpoint: std::env::var("OTEL_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string()),
        })
    }
}
