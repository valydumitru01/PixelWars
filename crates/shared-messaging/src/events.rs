/// NATS subject constants for routing domain events.
pub mod subjects {
    pub const AUTH_USER_REGISTERED: &str = "pixelwar.auth.user_registered";
    pub const AUTH_USER_LOGGED_IN: &str = "pixelwar.auth.user_logged_in";

    pub const CANVAS_PARCEL_CLAIMED: &str = "pixelwar.canvas.parcel_claimed";
    pub const CANVAS_PIXEL_UPDATED: &str = "pixelwar.canvas.pixel_updated";
    pub const CANVAS_SNAPSHOT: &str = "pixelwar.canvas.snapshot";

    pub const GROUP_CREATED: &str = "pixelwar.group.created";
    pub const GROUP_INVITE_SENT: &str = "pixelwar.group.invite_sent";
    pub const GROUP_INVITE_ACCEPTED: &str = "pixelwar.group.invite_accepted";
    pub const GROUP_DISBANDED: &str = "pixelwar.group.disbanded";

    pub const VOTING_WINDOW_OPENED: &str = "pixelwar.voting.window_opened";
    pub const VOTING_VOTE_CAST: &str = "pixelwar.voting.vote_cast";
    pub const VOTING_WINDOW_CLOSED: &str = "pixelwar.voting.window_closed";
    pub const VOTING_RESULTS: &str = "pixelwar.voting.results";

    pub const CHAT_MESSAGE: &str = "pixelwar.chat.message";

    pub const SCHEDULER_ACTIVITY_CHECK: &str = "pixelwar.scheduler.activity_check";
    pub const SCHEDULER_USER_DISQUALIFIED: &str = "pixelwar.scheduler.user_disqualified";
    pub const SCHEDULER_ROUND_STARTED: &str = "pixelwar.scheduler.round_started";
    pub const SCHEDULER_ROUND_ENDED: &str = "pixelwar.scheduler.round_ended";
}
