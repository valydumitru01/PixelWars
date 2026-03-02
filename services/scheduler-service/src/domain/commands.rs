use uuid::Uuid;

// Command structures for use case inputs - currently empty but available for future extension.
// Use cases may accept these commands to encapsulate their input parameters.

pub struct RunActivityCheckCommand {
    pub round_id: Uuid,
}

pub struct EndRoundCommand {
    pub round_id: Uuid,
}

pub struct StartRoundCommand;

pub struct CloseVotingCommand;
