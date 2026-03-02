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
    async fn vote_cast(&self, voter_id: Uuid, target_id: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::VOTING_VOTE_CAST,
                &DomainEvent::VoteCast { voter_id, target_id },
            )
            .await
    }
}
