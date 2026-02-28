use shared_common::errors::AppError;
use shared_common::models::parcel::ClaimParcelRequest;
use uuid::Uuid;

use super::contiguity::is_valid_rectangle;

/// Validate and process a parcel claim.
pub fn validate_claim(req: &ClaimParcelRequest) -> Result<(), AppError> {
    if !is_valid_rectangle(req.origin_x, req.origin_y, req.width, req.height) {
        return Err(AppError::Validation(format!(
            "Invalid parcel dimensions: {}x{} at ({},{}). Must be {} contiguous pixels within canvas bounds.",
            req.width, req.height, req.origin_x, req.origin_y,
            shared_common::models::parcel::PARCEL_PIXEL_COUNT
        )));
    }

    if req.description.is_empty() || req.description.len() > 500 {
        return Err(AppError::Validation(
            "Description must be between 1 and 500 characters".to_string(),
        ));
    }

    Ok(())
}
