use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tsa_core::{Result, TsaError};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub access_token_expiry: Duration,
    pub refresh_token_expiry: Duration,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            issuer: None,
            audience: None,
            access_token_expiry: Duration::minutes(15),
            refresh_token_expiry: Duration::days(7),
        }
    }

    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    pub fn with_access_expiry(mut self, duration: Duration) -> Self {
        self.access_token_expiry = duration;
        self
    }

    pub fn with_refresh_expiry(mut self, duration: Duration) -> Self {
        self.refresh_token_expiry = duration;
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
}

#[derive(Clone)]
pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl std::fmt::Debug for JwtManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtManager")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl JwtManager {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }

    pub fn generate_access_token(&self, user_id: Uuid) -> Result<String> {
        self.generate_token(user_id, self.config.access_token_expiry)
    }

    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String> {
        self.generate_token(user_id, self.config.refresh_token_expiry)
    }

    fn generate_token(&self, user_id: Uuid, expiry: Duration) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: (now + expiry).timestamp(),
            iat: now.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| TsaError::Internal(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::default();

        if let Some(ref issuer) = self.config.issuer {
            validation.set_issuer(&[issuer]);
        }

        if let Some(ref audience) = self.config.audience {
            validation.set_audience(&[audience]);
        }

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => TsaError::TokenExpired,
                _ => TsaError::InvalidToken,
            })
    }

    pub fn extract_user_id(&self, token: &str) -> Result<Uuid> {
        let claims = self.validate_token(token)?;
        Uuid::parse_str(&claims.sub).map_err(|_| TsaError::InvalidToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig::new("test_secret_key_for_testing")
    }

    fn test_manager() -> JwtManager {
        JwtManager::new(test_config())
    }

    #[test]
    fn test_jwt_config_defaults() {
        let config = JwtConfig::new("secret");
        assert_eq!(config.access_token_expiry, Duration::minutes(15));
        assert_eq!(config.refresh_token_expiry, Duration::days(7));
        assert!(config.issuer.is_none());
        assert!(config.audience.is_none());
    }

    #[test]
    fn test_jwt_config_builder() {
        let config = JwtConfig::new("secret")
            .with_issuer("my-app")
            .with_audience("my-audience")
            .with_access_expiry(Duration::hours(1))
            .with_refresh_expiry(Duration::days(30));

        assert_eq!(config.issuer, Some("my-app".to_string()));
        assert_eq!(config.audience, Some("my-audience".to_string()));
        assert_eq!(config.access_token_expiry, Duration::hours(1));
        assert_eq!(config.refresh_token_expiry, Duration::days(30));
    }

    #[test]
    fn test_generate_access_token() {
        let manager = test_manager();
        let user_id = Uuid::new_v4();

        let token = manager.generate_access_token(user_id).unwrap();
        assert!(!token.is_empty());
        assert!(token.contains('.'));
    }

    #[test]
    fn test_generate_refresh_token() {
        let manager = test_manager();
        let user_id = Uuid::new_v4();

        let token = manager.generate_refresh_token(user_id).unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_validate_valid_token() {
        let manager = test_manager();
        let user_id = Uuid::new_v4();

        let token = manager.generate_access_token(user_id).unwrap();
        let claims = manager.validate_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert!(claims.exp > Utc::now().timestamp());
    }

    #[test]
    fn test_validate_invalid_token() {
        let manager = test_manager();
        let result = manager.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_wrong_secret() {
        let manager1 = JwtManager::new(JwtConfig::new("secret1"));
        let manager2 = JwtManager::new(JwtConfig::new("secret2"));
        let user_id = Uuid::new_v4();

        let token = manager1.generate_access_token(user_id).unwrap();
        let result = manager2.validate_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_user_id() {
        let manager = test_manager();
        let user_id = Uuid::new_v4();

        let token = manager.generate_access_token(user_id).unwrap();
        let extracted = manager.extract_user_id(&token).unwrap();

        assert_eq!(user_id, extracted);
    }

    #[test]
    fn test_token_with_issuer_and_audience() {
        let config = JwtConfig::new("secret")
            .with_issuer("test-issuer")
            .with_audience("test-audience");
        let manager = JwtManager::new(config);
        let user_id = Uuid::new_v4();

        let token = manager.generate_access_token(user_id).unwrap();
        let claims = manager.validate_token(&token).unwrap();

        assert_eq!(claims.iss, Some("test-issuer".to_string()));
        assert_eq!(claims.aud, Some("test-audience".to_string()));
    }

    #[test]
    fn test_expired_token() {
        use jsonwebtoken::{encode, Header};

        let config = JwtConfig::new("secret");
        let manager = JwtManager::new(config);
        let user_id = Uuid::new_v4();

        let claims = Claims {
            sub: user_id.to_string(),
            exp: (Utc::now() - Duration::hours(1)).timestamp(),
            iat: (Utc::now() - Duration::hours(2)).timestamp(),
            iss: None,
            aud: None,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("secret".as_bytes()),
        )
        .unwrap();

        let result = manager.validate_token(&token);
        assert!(matches!(result, Err(TsaError::TokenExpired)));
    }

    #[test]
    fn test_different_expiry_different_tokens() {
        let config1 = JwtConfig::new("secret").with_access_expiry(Duration::hours(1));
        let config2 = JwtConfig::new("secret").with_access_expiry(Duration::hours(2));
        let manager1 = JwtManager::new(config1);
        let manager2 = JwtManager::new(config2);
        let user_id = Uuid::new_v4();

        let token1 = manager1.generate_access_token(user_id).unwrap();
        let token2 = manager2.generate_access_token(user_id).unwrap();

        assert_ne!(token1, token2);
    }
}
