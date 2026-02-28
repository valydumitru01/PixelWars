use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single pixel update on the canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelUpdate {
    pub parcel_id: Uuid,
    /// X position within the parcel (local coordinates).
    pub local_x: u32,
    /// Y position within the parcel (local coordinates).
    pub local_y: u32,
    /// RGBA color packed as u32.
    pub color: u32,
}

#[derive(Debug, Deserialize)]
pub struct BatchPixelUpdate {
    pub parcel_id: Uuid,
    pub pixels: Vec<PixelUpdate>,
}
