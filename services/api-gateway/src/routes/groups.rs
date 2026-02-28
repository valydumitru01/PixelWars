use axum::{routing::{get, post}, Router};

pub fn routes() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", post(create_group))
        .route("/:id", get(get_group))
        .route("/:id/invite", post(invite_member))
        .route("/:id/invite/accept", post(accept_invite))
}

async fn create_group() -> &'static str { "create_group" }
async fn get_group() -> &'static str { "get_group" }
async fn invite_member() -> &'static str { "invite_member" }
async fn accept_invite() -> &'static str { "accept_invite" }
