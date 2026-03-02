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
    async fn group_created(&self, group_id: Uuid, creator_id: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::GROUP_CREATED,
                &DomainEvent::GroupCreated {
                    group_id,
                    creator_id,
                },
            )
            .await
    }

    async fn invite_sent(&self, group_id: Uuid, from_user: Uuid, to_user: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::GROUP_INVITE_SENT,
                &DomainEvent::GroupInviteSent {
                    group_id,
                    from_user,
                    to_user,
                },
            )
            .await
    }

    async fn invite_accepted(&self, group_id: Uuid, user_id: Uuid) -> Result<()> {
        self.nats
            .publish(
                subjects::GROUP_INVITE_ACCEPTED,
                &DomainEvent::GroupInviteAccepted {
                    group_id,
                    user_id,
                },
            )
            .await
    }
}
