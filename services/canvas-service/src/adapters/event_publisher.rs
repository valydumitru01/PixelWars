use anyhow::Result;
use shared_common::events::DomainEvent;
use shared_messaging::NatsClient;
use shared_messaging::events::subjects;
use uuid::Uuid;

use crate::ports::EventPublisher;

pub struct NatsEventPublisher {
    nats: NatsClient,
}

impl NatsEventPublisher {
    pub fn new(nats: NatsClient) -> Self {
        Self { nats }
    }
}

#[tonic::async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn parcel_claimed(
        &self,
        user_id: Uuid,
        parcel_id: Uuid,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        self.nats
            .publish(
                subjects::CANVAS_PARCEL_CLAIMED,
                &DomainEvent::ParcelClaimed {
                    user_id,
                    parcel_id,
                    x,
                    y,
                    width,
                    height,
                },
            )
            .await
    }

    async fn pixels_updated(&self, parcel_id: Uuid, x: u32, y: u32, color: u32) -> Result<()> {
        self.nats
            .publish(
                subjects::CANVAS_PIXEL_UPDATED,
                &DomainEvent::PixelUpdated { parcel_id, x, y, color },
            )
            .await
    }
}
