use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use tsa_core::{Session, User};

pub struct AuthUser(pub User);

pub struct AuthSession(pub Session);

pub struct AuthUserSession {
    pub user: User,
    pub session: Session,
}

#[derive(Clone)]
pub struct SessionState {
    pub user: Option<User>,
    pub session: Option<Session>,
}

impl SessionState {
    pub fn user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.is_some() && self.session.is_some()
    }
}

pub enum AuthError {
    NoSession,
    InvalidSession,
    SessionExpired,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::NoSession => (StatusCode::UNAUTHORIZED, "No session found"),
            AuthError::InvalidSession => (StatusCode::UNAUTHORIZED, "Invalid session"),
            AuthError::SessionExpired => (StatusCode::UNAUTHORIZED, "Session expired"),
        };
        (status, message).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session_state = parts
            .extensions
            .get::<SessionState>()
            .ok_or(AuthError::NoSession)?;

        session_state
            .user
            .clone()
            .map(AuthUser)
            .ok_or(AuthError::NoSession)
    }
}

impl<S> FromRequestParts<S> for AuthSession
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session_state = parts
            .extensions
            .get::<SessionState>()
            .ok_or(AuthError::NoSession)?;

        session_state
            .session
            .clone()
            .map(AuthSession)
            .ok_or(AuthError::NoSession)
    }
}

impl<S> FromRequestParts<S> for AuthUserSession
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session_state = parts
            .extensions
            .get::<SessionState>()
            .ok_or(AuthError::NoSession)?;

        let user = session_state.user.clone().ok_or(AuthError::NoSession)?;
        let session = session_state.session.clone().ok_or(AuthError::NoSession)?;

        Ok(AuthUserSession { user, session })
    }
}

pub const DEFAULT_SESSION_COOKIE_NAME: &str = "tsa_session";

pub fn extract_session_token_from_cookie(jar: &CookieJar, cookie_name: &str) -> Option<String> {
    jar.get(cookie_name).map(|c| c.value().to_string())
}
