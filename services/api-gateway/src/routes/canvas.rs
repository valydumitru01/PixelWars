use axum::{routing::{get, post}, Router};

pub fn routes() -> Router<crate::state::AppState> {
    Router::new()
        .route("/parcels", post(claim_parcel))
        .route("/parcels/{id}", get(get_parcel))
        .route("/pixels", post(update_pixels))
        .route("/snapshot", get(get_snapshot))
}

async fn claim_parcel() -> &'static str { "claim_parcel" }
async fn get_parcel() -> &'static str { "get_parcel" }
async fn update_pixels() -> &'static str { "update_pixels" }
async fn get_snapshot() -> &'static str { "get_snapshot" }
