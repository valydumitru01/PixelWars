use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain events published over NATS for inter-service communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    // Auth events
    UserRegistered { user_id: Uuid, username: String },
    UserLoggedIn { user_id: Uuid },

    // Canvas events
    ParcelClaimed { user_id: Uuid, parcel_id: Uuid, x: u32, y: u32, width: u32, height: u32 },
    PixelUpdated { parcel_id: Uuid, x: u32, y: u32, color: u32 },
    CanvasSnapshotRequested { round_id: Uuid },

    // Group events
    GroupCreated { group_id: Uuid, creator_id: Uuid },
    GroupInviteSent { group_id: Uuid, from_user: Uuid, to_user: Uuid },
    GroupInviteAccepted { group_id: Uuid, user_id: Uuid },
    GroupDisbanded { group_id: Uuid },

    // Voting events
    VotingWindowOpened { round_id: Uuid },
    VoteCast { voter_id: Uuid, target_id: Uuid },
    VotingWindowClosed { round_id: Uuid },
    ResultsPublished { round_id: Uuid, winner_id: Uuid },

    // Chat events
    ChatMessage { channel: String, sender_id: Uuid, content: String },

    // Scheduler events
    ActivityCheckTriggered { round_id: Uuid },
    UserDisqualified { user_id: Uuid, round_id: Uuid, reason: String },
    RoundStarted { round_id: Uuid },
    RoundEnded { round_id: Uuid },
}
