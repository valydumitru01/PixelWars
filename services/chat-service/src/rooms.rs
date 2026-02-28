use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Manages active chat rooms and their subscribers.
#[derive(Clone, Default)]
pub struct RoomManager {
    /// Map of channel_id -> set of connected user_ids.
    rooms: Arc<RwLock<HashMap<String, HashSet<Uuid>>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn join(&self, channel: &str, user_id: Uuid) {
        let mut rooms = self.rooms.write().await;
        rooms
            .entry(channel.to_string())
            .or_default()
            .insert(user_id);
    }

    pub async fn leave(&self, channel: &str, user_id: &Uuid) {
        let mut rooms = self.rooms.write().await;
        if let Some(members) = rooms.get_mut(channel) {
            members.remove(user_id);
            if members.is_empty() {
                rooms.remove(channel);
            }
        }
    }

    pub async fn members(&self, channel: &str) -> HashSet<Uuid> {
        let rooms = self.rooms.read().await;
        rooms.get(channel).cloned().unwrap_or_default()
    }
}
