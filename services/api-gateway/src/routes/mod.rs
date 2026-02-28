pub mod auth;
pub mod canvas;
pub mod chat;
pub mod voting;
pub mod groups;

use axum::Router;
use crate::state::AppState;

/// Merge all route modules into the top-level API router.
pub fn api_routes(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/auth", auth::routes())
        .nest("/api/v1/canvas", canvas::routes())
        .nest("/api/v1/chat", chat::routes())
        .nest("/api/v1/voting", voting::routes())
        .nest("/api/v1/groups", groups::routes())
        .with_state(state)
}
