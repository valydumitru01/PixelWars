use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: String,
    version: &'static str,
}

/// Create health-check routes: GET /health and GET /ready.
pub fn health_routes(service_name: &str) -> Router {
    let name = service_name.to_string();
    Router::new()
        .route("/health", get({
            let name = name;
            move || async move {
                Json(HealthResponse {
                    status: "ok",
                    service: name,
                    version: env!("CARGO_PKG_VERSION"),
                })
            }
        }))
        .route("/ready", get(|| async { "ready" }))
}
