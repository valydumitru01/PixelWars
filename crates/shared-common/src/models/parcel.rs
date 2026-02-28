use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A contiguous region of pixels claimed by a user.
/// Each user selects exactly 10,000 topologically contiguous pixels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parcel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub round_id: Uuid,
    /// Top-left X coordinate on the 10k×10k canvas.
    pub origin_x: u32,
    /// Top-left Y coordinate on the 10k×10k canvas.
    pub origin_y: u32,
    /// Width in pixels (width × height must equal 10,000).
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// User's description of their planned drawing.
    pub description: String,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ClaimParcelRequest {
    pub origin_x: u32,
    pub origin_y: u32,
    pub width: u32,
    pub height: u32,
    pub description: String,
}

/// Represents a set of arbitrary contiguous pixel positions (for non-rectangular claims).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelRegion {
    /// List of (x, y) coordinates making up the region.
    pub pixels: Vec<(u32, u32)>,
}

pub const CANVAS_SIZE: u32 = 10_000;
pub const PARCEL_PIXEL_COUNT: u32 = 10_000;
