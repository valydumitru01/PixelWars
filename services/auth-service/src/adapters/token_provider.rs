use anyhow::Result;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use shared_common::models::user::UserClaims;
use uuid::Uuid;

use crate::ports::TokenProvider;

const TOKEN_EXPIRY_HOURS: i64 = 24;

pub struct JwtTokenProvider {
    secret: String,
}

impl JwtTokenProvider {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl TokenProvider for JwtTokenProvider {
    fn create_token(&self, user_id: Uuid, username: &str) -> Result<String> {
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
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;
        Ok(token)
    }

    fn validate_token(&self, token: &str) -> Result<UserClaims> {
        let data = decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }
}
