use axum::{extract::State, routing::post, Json, Router};
use shared_common::errors::AppError;
use crate::clients::auth::{LoginRequest, RegisterRequest};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

#[derive(serde::Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct AuthReplyJson {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<AuthReplyJson>, AppError> {
    let mut client = state.auth_client.clone();
    let response = client
        .register(RegisterRequest {
            username: body.username,
            email: body.email,
            password: body.password,
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(AuthReplyJson {
        token: response.token,
        user_id: response.user_id,
        username: response.username,
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Json<AuthReplyJson>, AppError> {
    let mut client = state.auth_client.clone();
    let response = client
        .login(LoginRequest {
            email: body.email,
            password: body.password,
        })
        .await
        .map_err(|e| AppError::Grpc(e.message().to_string()))?
        .into_inner();

    Ok(Json(AuthReplyJson {
        token: response.token,
        user_id: response.user_id,
        username: response.username,
    }))
}
