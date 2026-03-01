pub mod auth;
pub mod canvas;
pub mod chat;
pub mod voting;
pub mod groups;

use axum::Router;
use axum::routing::get;
use crate::state::AppState;
use crate::websocket;

/// Private routes requiring JWT authentication.
pub fn private_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/canvas",  canvas::routes())
        .nest("/api/v1/chat",    chat::routes())
        .nest("/api/v1/voting",  voting::routes())
        .nest("/api/v1/groups",  groups::routes())
        .route("/ws", get(websocket::handler::ws_handler))
}

/// Public routes (no JWT required).
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/auth", auth::routes())
}
