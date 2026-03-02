use std::sync::Arc;

use crate::application::{accept_invite, create_group, get_group, get_user_group, send_invite};

#[derive(Clone)]
pub struct GroupState {
    pub create_group: Arc<create_group::CreateGroup>,
    pub get_group: Arc<get_group::GetGroup>,
    pub send_invite: Arc<send_invite::SendInvite>,
    pub accept_invite: Arc<accept_invite::AcceptInvite>,
    pub get_user_group: Arc<get_user_group::GetUserGroup>,
}
