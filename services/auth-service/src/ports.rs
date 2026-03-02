use anyhow::Result;
use chrono::{DateTime, Utc};
use shared_common::models::user::UserClaims;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain data returned by the user repository
// ---------------------------------------------------------------------------

/// Minimal user row needed by the login/register use cases.
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_disqualified: bool,
    pub last_draw_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Outbound port: UserRepository
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait UserRepository: Send + Sync {
    /// Return true if a user with the given email OR username already exists.
    async fn exists_by_email_or_username(&self, email: &str, username: &str) -> Result<bool>;

    /// Insert a new user row and return its UUID.
    async fn create(&self, id: Uuid, username: &str, email: &str, password_hash: &str) -> Result<()>;

    /// Fetch a user by email (only active users).
    async fn find_by_email(&self, email: &str) -> Result<Option<UserRow>>;

    /// Fetch a user by id.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRow>>;
}

// ---------------------------------------------------------------------------
// Outbound port: PasswordHasher
// ---------------------------------------------------------------------------

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String>;
    fn verify(&self, password: &str, hash: &str) -> Result<bool>;
}

// ---------------------------------------------------------------------------
// Outbound port: TokenProvider
// ---------------------------------------------------------------------------

pub trait TokenProvider: Send + Sync {
    fn create_token(&self, user_id: Uuid, username: &str) -> Result<String>;
    fn validate_token(&self, token: &str) -> Result<UserClaims>;
}

// ---------------------------------------------------------------------------
// Outbound port: EventPublisher
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn user_registered(&self, user_id: Uuid, username: &str) -> Result<()>;
    async fn user_logged_in(&self, user_id: Uuid) -> Result<()>;
}
