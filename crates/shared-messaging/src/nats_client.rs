use async_nats::Client;
use shared_common::events::DomainEvent;
use tracing::{info, error};

/// Wrapper around the NATS client for publishing/subscribing to domain events.
#[derive(Clone)]
pub struct NatsClient {
    client: Client,
}

impl NatsClient {
    pub async fn connect(nats_url: &str) -> Result<Self, async_nats::ConnectError> {
        let client = async_nats::connect(nats_url).await?;
        info!("Connected to NATS at {}", nats_url);
        Ok(Self { client })
    }

    /// Publish a domain event to a subject.
    pub async fn publish(&self, subject: &str, event: &DomainEvent) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(event)?;
        self.client
            .publish(subject.to_string(), payload.into())
            .await?;
        Ok(())
    }

    /// Subscribe to a subject and receive domain events.
    pub async fn subscribe(
        &self,
        subject: &str,
    ) -> Result<async_nats::Subscriber, async_nats::SubscribeError> {
        let sub = self.client.subscribe(subject.to_string()).await?;
        info!(subject, "Subscribed to NATS subject");
        Ok(sub)
    }

    /// Get a reference to the inner NATS client.
    pub fn inner(&self) -> &Client {
        &self.client
    }
}
