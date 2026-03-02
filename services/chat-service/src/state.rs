use std::sync::Arc;

use crate::application::{send_message, get_messages};

#[derive(Clone)]
pub struct ChatState {
    pub send_message: Arc<send_message::SendMessage>,
    pub get_messages: Arc<get_messages::GetMessages>,
}
