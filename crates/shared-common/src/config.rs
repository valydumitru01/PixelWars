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
    pub fn from_env(service_name: &str) -> Result<Self, anyhow::Error> {
        // Generate the prefix (e.g., "api-gateway" -> "API_GATEWAY")
        let env_prefix = service_name.to_uppercase().replace("-", "_");

        // Helper to fetch and provide clear error messages
        let get_env = |key: &str| {
            std::env::var(key).map_err(|_| {
                anyhow::anyhow!("Missing required environment variable: {}", key)
            })
        };

        // Construct service-specific keys
        let port_key = format!("{}_PORT", env_prefix);

        Ok(Self {
            service_name: service_name.to_string(),

            host: get_env("HOST")?,

            port: get_env(&port_key)?.parse::<u16>()
                .map_err(|_| anyhow::anyhow!("{} must be a valid number", port_key))?,

            database_url: get_env("DATABASE_URL")?,

            redis_url: get_env("REDIS_URL")?,

            nats_url: get_env("NATS_URL")?,

            jwt_secret: get_env("JWT_SECRET")?,

            otel_endpoint: get_env("OTEL_ENDPOINT")?,
        })
    }
}