use anyhow::Result;
use shared_common::events::DomainEvent;
use shared_messaging::NatsClient;
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
    async fn message_sent(
        &self,
        channel_type: &str,
        channel_id: Option<Uuid>,
        sender_id: Uuid,
        content: &str,
    ) -> Result<()> {
        // Resolve the NATS subject based on channel type
        let nats_subject = match channel_type {
            "global" => "pixelwar.chat.global".to_string(),
            "group" => format!(
                "pixelwar.chat.group.{}",
                channel_id
                    .map(|id| id.to_string())
                    .unwrap_or_default()
            ),
            "whisper" => format!(
                "pixelwar.chat.whisper.{}",
                channel_id
                    .map(|id| id.to_string())
                    .unwrap_or_default()
            ),
            _ => "pixelwar.chat.global".to_string(),
        };

        let event = DomainEvent::ChatMessage {
            channel: nats_subject.clone(),
            sender_id,
            content: content.to_string(),
        };

        self.nats.publish(&nats_subject, &event).await
    }
}
