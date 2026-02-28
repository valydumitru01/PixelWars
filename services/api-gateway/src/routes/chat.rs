use axum::{routing::{get, post}, Router};

pub fn routes() -> Router<crate::state::AppState> {
    Router::new()
        .route("/messages", post(send_message))
        .route("/messages/:channel", get(get_messages))
}

async fn send_message() -> &'static str { "send_message" }
async fn get_messages() -> &'static str { "get_messages" }
