use axum::{routing::{get, post}, Router};

pub fn routes() -> Router<crate::state::AppState> {
    Router::new()
        .route("/vote", post(cast_vote))
        .route("/results", get(get_results))
}

async fn cast_vote() -> &'static str { "cast_vote" }
async fn get_results() -> &'static str { "get_results" }
