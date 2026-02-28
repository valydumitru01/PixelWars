use axum::{routing::post, Router};

pub fn routes() -> Router<crate::state::AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

async fn register() -> &'static str {
    // TODO: Proxy to auth-service via gRPC or HTTP
    "register"
}

async fn login() -> &'static str {
    // TODO: Proxy to auth-service
    "login"
}
