use std::sync::Arc;

use crate::application::{get_user, login_user, register_user, validate_token};

#[derive(Clone)]
pub struct AuthState {
    pub register_user: Arc<register_user::RegisterUser>,
    pub login_user: Arc<login_user::LoginUser>,
    pub validate_token: Arc<validate_token::ValidateToken>,
    pub get_user: Arc<get_user::GetUser>,
}
