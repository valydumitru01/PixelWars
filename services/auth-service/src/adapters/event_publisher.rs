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
    async fn user_registered(&self, user_id: Uuid, username: &str) -> Result<()> {
        self.nats
            .publish(
                subjects::AUTH_USER_REGISTERED,
                &DomainEvent::UserRegistered {
                    user_id,
                    username: username.to_string(),
                },
            )
            .await
    }

    async fn user_logged_in(&self, user_id: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::AUTH_USER_LOGGED_IN,
                &DomainEvent::UserLoggedIn { user_id },
            )
            .await
    }
}
