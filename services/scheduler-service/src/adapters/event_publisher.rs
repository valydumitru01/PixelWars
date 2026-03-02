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

#[async_trait::async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn user_disqualified(&self, user_id: Uuid, round_id: Uuid, reason: &str) -> Result<()> {
        self.nats
            .publish(
                subjects::SCHEDULER_USER_DISQUALIFIED,
                &DomainEvent::UserDisqualified {
                    user_id,
                    round_id,
                    reason: reason.to_string(),
                },
            )
            .await
    }

    async fn round_started(&self, round_id: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::SCHEDULER_ROUND_STARTED,
                &DomainEvent::RoundStarted { round_id },
            )
            .await
    }

    async fn round_ended(&self, round_id: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::SCHEDULER_ROUND_ENDED,
                &DomainEvent::RoundEnded { round_id },
            )
            .await
    }

    async fn voting_opened(&self, round_id: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::VOTING_WINDOW_OPENED,
                &DomainEvent::VotingWindowOpened { round_id },
            )
            .await
    }

    async fn voting_closed(&self, round_id: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::VOTING_WINDOW_CLOSED,
                &DomainEvent::VotingWindowClosed { round_id },
            )
            .await
    }
}
