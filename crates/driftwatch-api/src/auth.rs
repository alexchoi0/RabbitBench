use std::sync::Arc;
use tsa::{Auth, NoopCallbacks};
use tsa_adapter_seaorm::SeaOrmAdapter;
use tsa_core::{ApiKey, Session, User};

pub type TsaAuth = Auth<SeaOrmAdapter, NoopCallbacks>;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
    pub session: Option<Session>,
    pub api_key: Option<ApiKey>,
    pub token: String,
}

impl AuthUser {
    pub fn user_id(&self) -> uuid::Uuid {
        self.user.id
    }

    pub fn is_session_auth(&self) -> bool {
        self.session.is_some()
    }
}

#[derive(Debug)]
pub struct AuthError(pub String);

pub async fn validate_token(token: &str, auth: &Arc<TsaAuth>) -> Result<AuthUser, AuthError> {
    if let Ok((user, session)) = auth.validate_session(token).await {
        return Ok(AuthUser {
            user,
            session: Some(session),
            api_key: None,
            token: token.to_string(),
        });
    }

    if let Ok((api_key, user)) = auth.validate_api_key(token).await {
        return Ok(AuthUser {
            user,
            session: None,
            api_key: Some(api_key),
            token: token.to_string(),
        });
    }

    Err(AuthError("Invalid token".to_string()))
}
