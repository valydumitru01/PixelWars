use uuid::Uuid;

/// Input DTO for creating a group.
pub struct CreateGroupCommand {
    pub name: String,
    pub creator_id: Uuid,
    pub round_id: Uuid,
}

/// Input DTO for sending an invite.
pub struct SendInviteCommand {
    pub group_id: Uuid,
    pub from_user: Uuid,
    pub to_user: Uuid,
}

/// Input DTO for accepting an invite.
pub struct AcceptInviteCommand {
    pub invite_id: Uuid,
    pub user_id: Uuid,
}
