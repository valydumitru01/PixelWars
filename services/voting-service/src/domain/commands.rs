use uuid::Uuid;

/// Input DTO for the cast vote use case.
pub struct CastVoteCommand {
    pub round_id: Uuid,
    pub voter_id: Uuid,
    pub target_id: Uuid,
    pub target_type: String, // "parcel" or "group"
}

/// Input DTO for the get results use case.
pub struct GetResultsQuery {
    pub round_id: Uuid,
}
