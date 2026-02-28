use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use shared_common::models::UserClaims;
use uuid::Uuid;

const TOKEN_EXPIRY_HOURS: i64 = 24;

pub fn create_token(user_id: Uuid, username: &str, secret: &str) -> anyhow::Result<String> {
    let now = Utc::now();
    let claims = UserClaims {
        sub: user_id,
        username: username.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(TOKEN_EXPIRY_HOURS)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn validate_token(token: &str, secret: &str) -> anyhow::Result<UserClaims> {
    let token_data = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
