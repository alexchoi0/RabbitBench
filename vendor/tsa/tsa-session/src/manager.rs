use chrono::{Duration, Utc};
use std::sync::Arc;
use tsa_core::{Result, Session, SessionRepository, TsaError};
use tsa_token::OpaqueToken;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub session_expiry: Duration,
    pub token_length: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_expiry: Duration::days(30),
            token_length: 32,
        }
    }
}

pub struct SessionManager<R: SessionRepository> {
    repository: Arc<R>,
    config: SessionConfig,
}

impl<R: SessionRepository> SessionManager<R> {
    pub fn new(repository: Arc<R>, config: SessionConfig) -> Self {
        Self { repository, config }
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(Session, String)> {
        let (token, token_hash) = OpaqueToken::generate_with_hash(self.config.token_length)?;

        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            expires_at: now + self.config.session_expiry,
            created_at: now,
            ip_address,
            user_agent,
        };

        let created = self.repository.create(&session).await?;
        Ok((created, token))
    }

    pub async fn validate_session(&self, token: &str) -> Result<Session> {
        let token_hash = OpaqueToken::hash(token);
        let session = self
            .repository
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(TsaError::SessionNotFound)?;

        if session.expires_at < Utc::now() {
            self.repository.delete(session.id).await?;
            return Err(TsaError::SessionExpired);
        }

        Ok(session)
    }

    pub async fn invalidate_session(&self, session_id: Uuid) -> Result<()> {
        self.repository.delete(session_id).await
    }

    pub async fn invalidate_all_sessions(&self, user_id: Uuid) -> Result<()> {
        self.repository.delete_by_user_id(user_id).await
    }

    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        self.repository.find_by_user_id(user_id).await
    }

    pub async fn cleanup_expired(&self) -> Result<u64> {
        self.repository.delete_expired().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();
        assert_eq!(config.session_expiry, Duration::days(30));
        assert_eq!(config.token_length, 32);
    }

    #[test]
    fn test_session_config_custom() {
        let config = SessionConfig {
            session_expiry: Duration::hours(1),
            token_length: 64,
        };
        assert_eq!(config.session_expiry, Duration::hours(1));
        assert_eq!(config.token_length, 64);
    }
}
