use std::sync::Arc;

use crate::application::{claim_parcel, get_parcel, update_pixels, get_snapshot, check_overlap};

#[derive(Clone)]
pub struct CanvasState {
    pub claim_parcel: Arc<claim_parcel::ClaimParcel>,
    pub get_parcel: Arc<get_parcel::GetParcel>,
    pub update_pixels: Arc<update_pixels::UpdatePixels>,
    pub get_snapshot: Arc<get_snapshot::GetSnapshot>,
    pub check_overlap: Arc<check_overlap::CheckOverlap>,
}
