use std::net::{IpAddr, SocketAddr};
use serde::Deserialize;

/// Base configuration shared across all services.
/// Every field is read from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub service_name: String,
    /// Validated as a proper IP address at startup — never a bare hostname.
    pub host: IpAddr,
    pub port: u16,
    /// Port on which this service exposes its gRPC endpoint.
    pub grpc_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub nats_url: String,
    pub jwt_secret: String,
    pub otel_endpoint: String,
    // Downstream service gRPC addresses (used by the api-gateway and inter-service calls)
    pub auth_grpc_url: String,
    pub canvas_grpc_url: String,
    pub chat_grpc_url: String,
    pub voting_grpc_url: String,
    pub group_grpc_url: String,
}

impl ServiceConfig {
    pub fn from_env(service_name: &str) -> Result<Self, anyhow::Error> {
        // HOST must be a valid IP address (not a hostname) — validated here so
        // http_addr() / grpc_addr() are infallible.
        let host = std::env::var("HOST")
            .unwrap_or_else(|_| "0.0.0.0".into())
            .parse::<IpAddr>()
            .map_err(|e| anyhow::anyhow!("HOST is not a valid IP address: {e}"))?;

        // JWT_SECRET has no fallback — a missing secret in any environment means
        // token validation would silently use an empty key, which is a critical
        // security hole.
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| anyhow::anyhow!("JWT_SECRET env var must be set"))?;

        Ok(Self {
            service_name: service_name.to_string(),
            host,
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()?,
            grpc_port: std::env::var("GRPC_PORT")
                .unwrap_or_else(|_| "9000".into())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://pixelwar:pixelwar@localhost:5432/pixelwar".into()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://127.0.0.1:4222".into()),
            jwt_secret,
            otel_endpoint: std::env::var("OTEL_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".into()),
            auth_grpc_url: std::env::var("AUTH_GRPC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9001".into()),
            canvas_grpc_url: std::env::var("CANVAS_GRPC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9002".into()),
            chat_grpc_url: std::env::var("CHAT_GRPC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9003".into()),
            voting_grpc_url: std::env::var("VOTING_GRPC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9004".into()),
            group_grpc_url: std::env::var("GROUP_GRPC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9005".into()),
        })
    }

    /// Infallible — HOST is validated as an IpAddr in from_env().
    pub fn http_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }

    /// Infallible — HOST is validated as an IpAddr in from_env().
    pub fn grpc_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.grpc_port)
    }
}
