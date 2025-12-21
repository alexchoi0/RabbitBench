use chrono::{Duration, Utc};
use std::sync::Arc;
use tsa_core::{
    Adapter, Passkey, PasskeyChallenge, PasskeyChallengeRepository, PasskeyChallengeType,
    PasskeyRepository, Result, TsaError, User, UserRepository,
};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::AuthCallbacks;

pub struct PasskeyConfig {
    pub rp_id: String,
    pub rp_name: String,
    pub rp_origin: Url,
    pub challenge_timeout_seconds: i64,
}

impl Default for PasskeyConfig {
    fn default() -> Self {
        Self {
            rp_id: "localhost".to_string(),
            rp_name: "TSA Application".to_string(),
            rp_origin: Url::parse("http://localhost:3000").unwrap(),
            challenge_timeout_seconds: 300,
        }
    }
}

pub struct PasskeyManager {
    webauthn: Arc<Webauthn>,
    config: PasskeyConfig,
}

impl PasskeyManager {
    pub fn new(config: PasskeyConfig) -> Result<Self> {
        let builder = WebauthnBuilder::new(&config.rp_id, &config.rp_origin)
            .map_err(|e| TsaError::Configuration(e.to_string()))?
            .rp_name(&config.rp_name);

        let webauthn = Arc::new(
            builder
                .build()
                .map_err(|e| TsaError::Configuration(e.to_string()))?,
        );

        Ok(Self { webauthn, config })
    }

    pub fn webauthn(&self) -> &Webauthn {
        &self.webauthn
    }

    pub fn challenge_timeout(&self) -> i64 {
        self.config.challenge_timeout_seconds
    }
}

pub struct PasskeyRegistrationStart {
    pub challenge_id: Uuid,
    pub options: CreationChallengeResponse,
}

pub struct PasskeyAuthenticationStart {
    pub challenge_id: Uuid,
    pub options: RequestChallengeResponse,
}

impl<A: Adapter, C: AuthCallbacks> crate::Auth<A, C> {
    pub async fn start_passkey_registration(
        &self,
        user_id: Uuid,
        passkey_manager: &PasskeyManager,
    ) -> Result<PasskeyRegistrationStart> {
        let user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let existing_passkeys = self.adapter.passkeys().find_by_user(user_id).await?;
        let exclude_credentials: Vec<CredentialID> = existing_passkeys
            .iter()
            .map(|p| CredentialID::from(p.credential_id.clone()))
            .collect();

        let display_name = user.name.clone().unwrap_or_else(|| user.email.clone());

        let (ccr, reg_state) = passkey_manager
            .webauthn()
            .start_passkey_registration(
                user_id,
                &user.email,
                &display_name,
                Some(exclude_credentials),
            )
            .map_err(|e| TsaError::Internal(e.to_string()))?;

        let state_bytes =
            serde_json::to_vec(&reg_state).map_err(|e| TsaError::Internal(e.to_string()))?;

        let challenge_bytes: Vec<u8> = ccr.public_key.challenge.as_ref().to_vec();

        let now = Utc::now();
        let challenge = PasskeyChallenge {
            id: Uuid::new_v4(),
            user_id: Some(user_id),
            challenge: challenge_bytes.to_vec(),
            challenge_type: PasskeyChallengeType::Registration,
            state: state_bytes,
            expires_at: now + Duration::seconds(passkey_manager.challenge_timeout()),
            created_at: now,
        };

        self.adapter.passkey_challenges().create(&challenge).await?;

        Ok(PasskeyRegistrationStart {
            challenge_id: challenge.id,
            options: ccr,
        })
    }

    pub async fn complete_passkey_registration(
        &self,
        challenge_id: Uuid,
        response: RegisterPublicKeyCredential,
        passkey_name: &str,
        passkey_manager: &PasskeyManager,
    ) -> Result<Passkey> {
        let challenge = self
            .adapter
            .passkey_challenges()
            .find_by_id(challenge_id)
            .await?
            .ok_or(TsaError::PasskeyChallengeNotFound)?;

        if challenge.expires_at < Utc::now() {
            self.adapter
                .passkey_challenges()
                .delete(challenge_id)
                .await?;
            return Err(TsaError::PasskeyChallengeExpired);
        }

        if challenge.challenge_type != PasskeyChallengeType::Registration {
            return Err(TsaError::InvalidInput("Invalid challenge type".to_string()));
        }

        let user_id = challenge.user_id.ok_or(TsaError::UserNotFound)?;

        let reg_state: PasskeyRegistration = serde_json::from_slice(&challenge.state)
            .map_err(|e| TsaError::Internal(e.to_string()))?;

        let passkey_credential = passkey_manager
            .webauthn()
            .finish_passkey_registration(&response, &reg_state)
            .map_err(|_| TsaError::PasskeyVerificationFailed)?;

        self.adapter
            .passkey_challenges()
            .delete(challenge_id)
            .await?;

        let now = Utc::now();
        let passkey = Passkey {
            id: Uuid::new_v4(),
            user_id,
            credential_id: passkey_credential.cred_id().to_vec(),
            public_key: serde_json::to_vec(&passkey_credential)
                .map_err(|e| TsaError::Internal(e.to_string()))?,
            counter: 0,
            name: passkey_name.to_string(),
            transports: None,
            created_at: now,
            last_used_at: None,
        };

        self.adapter.passkeys().create(&passkey).await
    }

    pub async fn start_passkey_authentication(
        &self,
        email: &str,
        passkey_manager: &PasskeyManager,
    ) -> Result<PasskeyAuthenticationStart> {
        let user = self
            .adapter
            .users()
            .find_by_email(email)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let passkeys = self.adapter.passkeys().find_by_user(user.id).await?;

        if passkeys.is_empty() {
            return Err(TsaError::PasskeyNotFound);
        }

        let credentials: Vec<webauthn_rs::prelude::Passkey> = passkeys
            .iter()
            .filter_map(|p| serde_json::from_slice(&p.public_key).ok())
            .collect();

        if credentials.is_empty() {
            return Err(TsaError::PasskeyNotFound);
        }

        let (rcr, auth_state) = passkey_manager
            .webauthn()
            .start_passkey_authentication(&credentials)
            .map_err(|e| TsaError::Internal(e.to_string()))?;

        let state_bytes =
            serde_json::to_vec(&auth_state).map_err(|e| TsaError::Internal(e.to_string()))?;

        let challenge_bytes: Vec<u8> = rcr.public_key.challenge.as_ref().to_vec();

        let now = Utc::now();
        let challenge = PasskeyChallenge {
            id: Uuid::new_v4(),
            user_id: Some(user.id),
            challenge: challenge_bytes.to_vec(),
            challenge_type: PasskeyChallengeType::Authentication,
            state: state_bytes,
            expires_at: now + Duration::seconds(passkey_manager.challenge_timeout()),
            created_at: now,
        };

        self.adapter.passkey_challenges().create(&challenge).await?;

        Ok(PasskeyAuthenticationStart {
            challenge_id: challenge.id,
            options: rcr,
        })
    }

    pub async fn complete_passkey_authentication(
        &self,
        challenge_id: Uuid,
        response: PublicKeyCredential,
        passkey_manager: &PasskeyManager,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, crate::core::Session, String)> {
        let challenge = self
            .adapter
            .passkey_challenges()
            .find_by_id(challenge_id)
            .await?
            .ok_or(TsaError::PasskeyChallengeNotFound)?;

        if challenge.expires_at < Utc::now() {
            self.adapter
                .passkey_challenges()
                .delete(challenge_id)
                .await?;
            return Err(TsaError::PasskeyChallengeExpired);
        }

        if challenge.challenge_type != PasskeyChallengeType::Authentication {
            return Err(TsaError::InvalidInput("Invalid challenge type".to_string()));
        }

        let user_id = challenge.user_id.ok_or(TsaError::UserNotFound)?;

        let auth_state: PasskeyAuthentication = serde_json::from_slice(&challenge.state)
            .map_err(|e| TsaError::Internal(e.to_string()))?;

        let auth_result = passkey_manager
            .webauthn()
            .finish_passkey_authentication(&response, &auth_state)
            .map_err(|_| TsaError::PasskeyVerificationFailed)?;

        self.adapter
            .passkey_challenges()
            .delete(challenge_id)
            .await?;

        let credential_id_bytes = response.id.as_ref();
        if let Some(mut passkey) = self
            .adapter
            .passkeys()
            .find_by_credential_id(credential_id_bytes)
            .await?
        {
            passkey.counter = auth_result.counter();
            passkey.last_used_at = Some(Utc::now());
            self.adapter.passkeys().update(&passkey).await?;
        }

        let user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let (session, token) = self
            .create_session_internal(&user, ip_address, user_agent)
            .await?;

        Ok((user, session, token))
    }

    pub async fn list_passkeys(&self, user_id: Uuid) -> Result<Vec<Passkey>> {
        self.adapter.passkeys().find_by_user(user_id).await
    }

    pub async fn delete_passkey(&self, user_id: Uuid, passkey_id: Uuid) -> Result<()> {
        let passkey = self
            .adapter
            .passkeys()
            .find_by_id(passkey_id)
            .await?
            .ok_or(TsaError::PasskeyNotFound)?;

        if passkey.user_id != user_id {
            return Err(TsaError::InsufficientPermissions);
        }

        self.adapter.passkeys().delete(passkey_id).await
    }

    pub async fn rename_passkey(
        &self,
        user_id: Uuid,
        passkey_id: Uuid,
        new_name: &str,
    ) -> Result<Passkey> {
        let mut passkey = self
            .adapter
            .passkeys()
            .find_by_id(passkey_id)
            .await?
            .ok_or(TsaError::PasskeyNotFound)?;

        if passkey.user_id != user_id {
            return Err(TsaError::InsufficientPermissions);
        }

        passkey.name = new_name.to_string();
        self.adapter.passkeys().update(&passkey).await
    }
}
