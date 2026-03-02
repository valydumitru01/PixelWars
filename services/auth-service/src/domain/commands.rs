/// Input DTO for the register use case.
pub struct RegisterCommand {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Input DTO for the login use case.
pub struct LoginCommand {
    pub email: String,
    pub password: String,
}
