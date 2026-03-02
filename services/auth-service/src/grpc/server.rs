use tonic::{Request, Response, Status};

use crate::domain::commands::{LoginCommand, RegisterCommand};
use crate::state::AuthState;

pub mod proto {
    tonic::include_proto!("pixelwar.auth");
}

use proto::auth_service_server::AuthService;
pub use proto::auth_service_server::AuthServiceServer;
use proto::*;

pub struct AuthGrpcService {
    pub state: AuthState,
}

#[tonic::async_trait]
impl AuthService for AuthGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthReply>, Status> {
        let req = request.into_inner();

        let cmd = RegisterCommand {
            username: req.username,
            email: req.email,
            password: req.password,
        };

        let output = self.state.register_user
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(AuthReply {
            token: output.token,
            user_id: output.user_id.to_string(),
            username: output.username,
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthReply>, Status> {
        let req = request.into_inner();

        let cmd = LoginCommand {
            email: req.email,
            password: req.password,
        };

        let output = self.state.login_user
            .execute(cmd)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(AuthReply {
            token: output.token,
            user_id: output.user_id.to_string(),
            username: output.username,
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenReply>, Status> {
        let token = request.into_inner().token;
        match self.state.validate_token.execute(&token) {
            Ok(claims) => Ok(Response::new(ValidateTokenReply {
                valid: true,
                user_id: claims.sub.to_string(),
                username: claims.username,
            })),
            Err(_) => Ok(Response::new(ValidateTokenReply {
                valid: false,
                user_id: String::new(),
                username: String::new(),
            })),
        }
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserReply>, Status> {
        let user_id: uuid::Uuid = request
            .into_inner()
            .user_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        let user = self.state.get_user
            .execute(user_id)
            .await
            .map_err(shared_common::errors::app_error_to_status)?;

        Ok(Response::new(UserReply {
            user_id: user.id.to_string(),
            username: user.username,
            email: user.email,
            is_disqualified: user.is_disqualified,
            last_draw_at: user
                .last_draw_at
                .map(|t| t.to_rfc3339())
                .unwrap_or_default(),
        }))
    }
}

