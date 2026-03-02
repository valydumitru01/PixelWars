use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain data returned by the parcel repository
// ---------------------------------------------------------------------------

pub struct ParcelRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub round_id: Uuid,
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: i32,
    pub height: i32,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Outbound port: ParcelRepository
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait ParcelRepository: Send + Sync {
    /// Find a parcel by its ID.
    async fn find_by_id(&self, parcel_id: Uuid) -> Result<Option<ParcelRow>>;

    /// Find all parcels claimed by a user in a specific round.
    async fn find_by_user_round(&self, user_id: Uuid, round_id: Uuid) -> Result<Option<ParcelRow>>;

    /// Check if a region overlaps with any existing parcel in a round.
    async fn check_overlap(
        &self,
        round_id: Uuid,
        origin_x: i32,
        origin_y: i32,
        width: i32,
        height: i32,
    ) -> Result<bool>;

    /// Create a new parcel.
    async fn create(
        &self,
        id: Uuid,
        user_id: Uuid,
        round_id: Uuid,
        origin_x: i32,
        origin_y: i32,
        width: i32,
        height: i32,
        description: &str,
    ) -> Result<()>;

    /// Update the parcel's updated_at timestamp.
    async fn update_parcel_timestamp(&self, parcel_id: Uuid, timestamp: DateTime<Utc>) -> Result<()>;

    /// Update the user's last_draw_at timestamp.
    async fn update_user_last_draw(&self, user_id: Uuid, timestamp: DateTime<Utc>) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Outbound port: PixelCache
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait PixelCache: Send + Sync {
    /// Set a pixel color at the given canvas coordinates.
    async fn set_pixel(&self, x: u32, y: u32, color: u32) -> Result<()>;

    /// Get a pixel color at the given canvas coordinates.
    async fn get_pixel(&self, x: u32, y: u32) -> Result<u32>;

    /// Get a rectangular region as raw RGBA bytes (Vec<u8> of u32 colors in big-endian).
    async fn get_snapshot_region(&self, x: u32, y: u32, width: u32, height: u32) -> Result<Vec<u8>>;
}

// ---------------------------------------------------------------------------
// Outbound port: EventPublisher
// ---------------------------------------------------------------------------

#[tonic::async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a parcel_claimed event.
    async fn parcel_claimed(
        &self,
        user_id: Uuid,
        parcel_id: Uuid,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<()>;

    /// Publish pixels_updated event.
    async fn pixels_updated(&self, parcel_id: Uuid, x: u32, y: u32, color: u32) -> Result<()>;
}
