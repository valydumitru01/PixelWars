use std::sync::Arc;

use shared_common::errors::AppError;
use uuid::Uuid;

use crate::ports::ParcelRepository;

/// Output returned from the get parcel use case.
pub struct GetParcelOutput {
    pub parcel_id: Uuid,
    pub user_id: Uuid,
    pub round_id: Uuid,
    pub origin_x: u32,
    pub origin_y: u32,
    pub width: u32,
    pub height: u32,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetParcel {
    parcel_repo: Arc<dyn ParcelRepository>,
}

impl GetParcel {
    pub fn new(parcel_repo: Arc<dyn ParcelRepository>) -> Self {
        Self { parcel_repo }
    }

    pub async fn execute(&self, parcel_id: Uuid) -> Result<GetParcelOutput, AppError> {
        let parcel = self.parcel_repo
            .find_by_id(parcel_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or(AppError::NotFound("Parcel not found".into()))?;

        Ok(GetParcelOutput {
            parcel_id: parcel.id,
            user_id: parcel.user_id,
            round_id: parcel.round_id,
            origin_x: parcel.origin_x as u32,
            origin_y: parcel.origin_y as u32,
            width: parcel.width as u32,
            height: parcel.height as u32,
            description: parcel.description,
            created_at: parcel.created_at,
        })
    }
}
