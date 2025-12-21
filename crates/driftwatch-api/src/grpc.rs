use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::auth::TsaAuth;

pub mod auth {
    tonic::include_proto!("driftwatch.auth");
}

use auth::auth_service_server::AuthService;
use auth::{
    ApiKey as ProtoApiKey, AuthResponse, CreateApiKeyRequest, CreateApiKeyResponse, GetMeRequest,
    GetMeResponse, RevokeApiKeyRequest, RevokeApiKeyResponse, SigninRequest, SignoutRequest,
    SignoutResponse, SignupRequest, User as ProtoUser, ValidateTokenRequest, ValidateTokenResponse,
};

pub struct AuthServiceImpl {
    pub auth: Arc<TsaAuth>,
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn signup(
        &self,
        request: Request<SignupRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let (user, _session, token) = self
            .auth
            .signup(&req.email, &req.password, req.name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AuthResponse {
            user: Some(user_to_proto(&user)),
            session_token: token,
        }))
    }

    async fn signin(
        &self,
        request: Request<SigninRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let (user, _session, token) = self
            .auth
            .signin(&req.email, &req.password, None, None)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        Ok(Response::new(AuthResponse {
            user: Some(user_to_proto(&user)),
            session_token: token,
        }))
    }

    async fn signout(
        &self,
        request: Request<SignoutRequest>,
    ) -> Result<Response<SignoutResponse>, Status> {
        let req = request.into_inner();

        self.auth
            .signout(&req.session_token)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SignoutResponse { success: true }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        if let Ok((user, _session)) = self.auth.validate_session(&req.token).await {
            return Ok(Response::new(ValidateTokenResponse {
                valid: true,
                user: Some(user_to_proto(&user)),
            }));
        }

        if let Ok((_api_key, user)) = self.auth.validate_api_key(&req.token).await {
            return Ok(Response::new(ValidateTokenResponse {
                valid: true,
                user: Some(user_to_proto(&user)),
            }));
        }

        Ok(Response::new(ValidateTokenResponse {
            valid: false,
            user: None,
        }))
    }

    async fn create_api_key(
        &self,
        request: Request<CreateApiKeyRequest>,
    ) -> Result<Response<CreateApiKeyResponse>, Status> {
        let req = request.into_inner();

        let (user, _session) = self
            .auth
            .validate_session(&req.session_token)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        let (api_key, secret) = self
            .auth
            .create_api_key(user.id, &req.name, req.scopes, None, None)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateApiKeyResponse {
            api_key: Some(api_key_to_proto(&api_key)),
            secret,
        }))
    }

    async fn revoke_api_key(
        &self,
        request: Request<RevokeApiKeyRequest>,
    ) -> Result<Response<RevokeApiKeyResponse>, Status> {
        let req = request.into_inner();

        let (user, _session) = self
            .auth
            .validate_session(&req.session_token)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        let api_key_id = uuid::Uuid::parse_str(&req.api_key_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        self.auth
            .delete_api_key(user.id, api_key_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RevokeApiKeyResponse { success: true }))
    }

    async fn get_me(
        &self,
        request: Request<GetMeRequest>,
    ) -> Result<Response<GetMeResponse>, Status> {
        let req = request.into_inner();

        if let Ok((user, _session)) = self.auth.validate_session(&req.token).await {
            return Ok(Response::new(GetMeResponse {
                user: Some(user_to_proto(&user)),
            }));
        }

        if let Ok((_api_key, user)) = self.auth.validate_api_key(&req.token).await {
            return Ok(Response::new(GetMeResponse {
                user: Some(user_to_proto(&user)),
            }));
        }

        Err(Status::unauthenticated("Invalid token"))
    }
}

fn user_to_proto(user: &tsa_core::User) -> ProtoUser {
    ProtoUser {
        id: user.id.to_string(),
        email: user.email.clone(),
        name: user.name.clone(),
        email_verified: user.email_verified,
    }
}

fn api_key_to_proto(api_key: &tsa_core::ApiKey) -> ProtoApiKey {
    ProtoApiKey {
        id: api_key.id.to_string(),
        name: api_key.name.clone(),
        prefix: api_key.prefix.clone(),
        scopes: api_key.scopes.clone(),
        expires_at: api_key.expires_at.map(|dt| dt.to_rfc3339()),
        created_at: api_key.created_at.to_rfc3339(),
    }
}
