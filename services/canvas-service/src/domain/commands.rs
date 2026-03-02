use uuid::Uuid;

/// Input DTO for the claim parcel use case.
pub struct ClaimParcelCommand {
    pub user_id: Uuid,
    pub round_id: Uuid,
    pub origin_x: u32,
    pub origin_y: u32,
    pub width: u32,
    pub height: u32,
    pub description: String,
}

/// A single pixel update in local parcel coordinates.
pub struct PixelUpdate {
    pub local_x: u32,
    pub local_y: u32,
    pub color: u32,
}

/// Input DTO for the update pixels use case.
pub struct UpdatePixelsCommand {
    pub parcel_id: Uuid,
    pub user_id: Uuid,
    pub pixels: Vec<PixelUpdate>,
}

/// Input DTO for the get snapshot use case.
pub struct SnapshotQuery {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Input DTO for the check overlap use case.
pub struct CheckOverlapQuery {
    pub round_id: Uuid,
    pub origin_x: u32,
    pub origin_y: u32,
    pub width: u32,
    pub height: u32,
}
