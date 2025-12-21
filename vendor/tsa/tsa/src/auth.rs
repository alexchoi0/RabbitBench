use std::sync::Arc;

use chrono::Utc;
use tsa_core::{
    Account, AccountRepository, Adapter, ApiKey, ApiKeyRepository, ApiKeyWebhookData,
    AuthWebhookData, InvitationStatus, InvitationWebhookData, MemberWebhookData, Organization,
    OrganizationInvitation, OrganizationInvitationRepository, OrganizationMember,
    OrganizationMemberRepository, OrganizationRepository, OrganizationRole,
    OrganizationWebhookData, Result, Session, SessionRepository, SessionWebhookData, TokenType,
    TsaError, TwoFactor, TwoFactorRepository, User, UserRepository, UserWebhookData,
    VerificationToken, VerificationTokenRepository, WebhookData, WebhookEvent, WebhookPayload,
};
use tsa_token::OpaqueToken;
use uuid::Uuid;

use crate::two_factor::{BackupCodes, TwoFactorMethod, TwoFactorSetup};
use crate::webhook::{NoopWebhookSender, WebhookSender};
use crate::{AuthCallbacks, AuthConfig, Password};

#[cfg(feature = "totp")]
use crate::two_factor::TotpManager;

pub struct Auth<A: Adapter, C: AuthCallbacks, W: WebhookSender = NoopWebhookSender> {
    pub(crate) adapter: A,
    pub(crate) config: AuthConfig,
    pub(crate) callbacks: C,
    pub(crate) webhooks: Arc<W>,
}

impl<A: Adapter, C: AuthCallbacks> Auth<A, C, NoopWebhookSender> {
    pub fn new(adapter: A, config: AuthConfig, callbacks: C) -> Self {
        Self {
            adapter,
            config,
            callbacks,
            webhooks: Arc::new(NoopWebhookSender),
        }
    }
}

impl<A: Adapter, C: AuthCallbacks, W: WebhookSender> Auth<A, C, W> {
    pub fn with_webhooks<W2: WebhookSender>(self, webhooks: W2) -> Auth<A, C, W2> {
        Auth {
            adapter: self.adapter,
            config: self.config,
            callbacks: self.callbacks,
            webhooks: Arc::new(webhooks),
        }
    }

    async fn send_webhook(&self, event: WebhookEvent, data: WebhookData) {
        let payload = WebhookPayload::new(event, data);
        if let Err(e) = self.webhooks.send(payload).await {
            tracing::warn!("Webhook delivery failed for {}: {}", event, e);
        }
    }

    pub async fn signup(
        &self,
        email: &str,
        password: &str,
        name: Option<String>,
    ) -> Result<(User, Session, String)> {
        if let Some(_existing) = self.adapter.users().find_by_email(email).await? {
            return Err(TsaError::UserAlreadyExists);
        }

        let now = Utc::now();
        let user_id = Uuid::new_v4();

        let user = User {
            id: user_id,
            email: email.to_string(),
            email_verified: false,
            phone: None,
            phone_verified: false,
            name,
            image: None,
            created_at: now,
            updated_at: now,
        };

        let user = self.adapter.users().create(&user).await?;

        let password_hash = Password::hash(password)?;
        self.create_credential_account(&user, &password_hash)
            .await?;

        let (session, token) = self.create_session_internal(&user, None, None).await?;

        self.callbacks.on_user_created(&user).await?;

        if self.config.require_email_verification {
            self.send_verification_email_internal(&user).await?;
        }

        self.send_webhook(
            WebhookEvent::UserCreated,
            WebhookData::User(UserWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                name: user.name.clone(),
            }),
        )
        .await;

        self.send_webhook(
            WebhookEvent::SessionCreated,
            WebhookData::Session(SessionWebhookData {
                session_id: session.id,
                user_id: user.id,
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
            }),
        )
        .await;

        Ok((user, session, token))
    }

    pub async fn signin(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, Session, String)> {
        let user = self
            .adapter
            .users()
            .find_by_email(email)
            .await?
            .ok_or(TsaError::InvalidCredentials)?;

        if self.config.require_email_verification && !user.email_verified {
            self.send_verification_email_internal(&user).await?;
            return Err(TsaError::EmailNotVerified);
        }

        let accounts = self.adapter.accounts().find_by_user_id(user.id).await?;
        let credential_account = accounts
            .iter()
            .find(|a| a.provider == "credential")
            .ok_or(TsaError::InvalidCredentials)?;

        let password_hash = self
            .get_password_hash(user.id)
            .await?
            .ok_or(TsaError::InvalidCredentials)?;

        if !Password::verify(password, &password_hash)? {
            return Err(TsaError::InvalidCredentials);
        }

        let (session, token) = self
            .create_session_internal(&user, ip_address, user_agent)
            .await?;

        self.send_webhook(
            WebhookEvent::SigninSuccess,
            WebhookData::Auth(AuthWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
                failure_reason: None,
            }),
        )
        .await;

        self.send_webhook(
            WebhookEvent::SessionCreated,
            WebhookData::Session(SessionWebhookData {
                session_id: session.id,
                user_id: user.id,
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
            }),
        )
        .await;

        Ok((user, session, token))
    }

    pub async fn signout(&self, session_token: &str) -> Result<()> {
        let token_hash = OpaqueToken::hash(session_token);
        if let Some(session) = self
            .adapter
            .sessions()
            .find_by_token_hash(&token_hash)
            .await?
        {
            self.adapter.sessions().delete(session.id).await?;

            self.send_webhook(
                WebhookEvent::SignoutSuccess,
                WebhookData::Session(SessionWebhookData {
                    session_id: session.id,
                    user_id: session.user_id,
                    ip_address: session.ip_address,
                    user_agent: session.user_agent,
                }),
            )
            .await;
        }
        Ok(())
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<(User, Session)> {
        let token_hash = OpaqueToken::hash(session_token);
        let mut session = self
            .adapter
            .sessions()
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(TsaError::SessionNotFound)?;

        let now = Utc::now();
        if session.expires_at < now {
            self.adapter.sessions().delete(session.id).await?;
            return Err(TsaError::SessionExpired);
        }

        if self.config.enable_session_refresh {
            let time_remaining = session.expires_at - now;
            if time_remaining < self.config.session_refresh_threshold {
                session.expires_at = now + self.config.session_expiry;
                session = self.adapter.sessions().update(&session).await?;
            }
        }

        let user = self
            .adapter
            .users()
            .find_by_id(session.user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        Ok((user, session))
    }

    pub async fn refresh_session(&self, session_token: &str) -> Result<Session> {
        let token_hash = OpaqueToken::hash(session_token);
        let mut session = self
            .adapter
            .sessions()
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(TsaError::SessionNotFound)?;

        if session.expires_at < Utc::now() {
            self.adapter.sessions().delete(session.id).await?;
            return Err(TsaError::SessionExpired);
        }

        session.expires_at = Utc::now() + self.config.session_expiry;
        let session = self.adapter.sessions().update(&session).await?;

        Ok(session)
    }

    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        self.adapter.sessions().find_by_user_id(user_id).await
    }

    pub async fn revoke_session(&self, session_id: Uuid) -> Result<()> {
        self.adapter.sessions().delete(session_id).await
    }

    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<()> {
        self.adapter.sessions().delete_by_user_id(user_id).await
    }

    pub async fn revoke_other_sessions(
        &self,
        user_id: Uuid,
        current_session_id: Uuid,
    ) -> Result<()> {
        let sessions = self.adapter.sessions().find_by_user_id(user_id).await?;
        for session in sessions {
            if session.id != current_session_id {
                self.adapter.sessions().delete(session.id).await?;
            }
        }
        Ok(())
    }

    pub async fn send_verification_email(&self, email: &str) -> Result<()> {
        let user = self
            .adapter
            .users()
            .find_by_email(email)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        if user.email_verified {
            return Ok(());
        }

        self.send_verification_email_internal(&user).await
    }

    pub async fn verify_email(&self, token: &str) -> Result<User> {
        let token_hash = OpaqueToken::hash(token);
        let verification = self
            .adapter
            .verification_tokens()
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(TsaError::InvalidToken)?;

        if verification.token_type != TokenType::EmailVerification {
            return Err(TsaError::InvalidToken);
        }

        if verification.expires_at < Utc::now() {
            self.adapter
                .verification_tokens()
                .delete(verification.id)
                .await?;
            return Err(TsaError::TokenExpired);
        }

        let mut user = self
            .adapter
            .users()
            .find_by_id(verification.user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        user.email_verified = true;
        user.updated_at = Utc::now();
        let user = self.adapter.users().update(&user).await?;

        self.adapter
            .verification_tokens()
            .delete(verification.id)
            .await?;

        self.send_webhook(
            WebhookEvent::EmailVerified,
            WebhookData::User(UserWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                name: user.name.clone(),
            }),
        )
        .await;

        Ok(user)
    }

    pub async fn request_password_reset(&self, email: &str) -> Result<()> {
        let user = match self.adapter.users().find_by_email(email).await? {
            Some(user) => user,
            None => return Ok(()),
        };

        let (token, token_hash) = OpaqueToken::generate_with_hash(32)?;
        let now = Utc::now();

        let verification = VerificationToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            token_hash,
            token_type: TokenType::PasswordReset,
            expires_at: now + self.config.password_reset_token_expiry,
            created_at: now,
        };

        self.adapter
            .verification_tokens()
            .create(&verification)
            .await?;

        self.callbacks
            .send_password_reset_email(&user, &token)
            .await?;

        self.send_webhook(
            WebhookEvent::PasswordResetRequested,
            WebhookData::User(UserWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                name: user.name.clone(),
            }),
        )
        .await;

        Ok(())
    }

    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<()> {
        let token_hash = OpaqueToken::hash(token);
        let verification = self
            .adapter
            .verification_tokens()
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(TsaError::InvalidToken)?;

        if verification.token_type != TokenType::PasswordReset {
            return Err(TsaError::InvalidToken);
        }

        if verification.expires_at < Utc::now() {
            self.adapter
                .verification_tokens()
                .delete(verification.id)
                .await?;
            return Err(TsaError::TokenExpired);
        }

        let password_hash = Password::hash(new_password)?;
        self.update_password_hash(verification.user_id, &password_hash)
            .await?;

        self.adapter
            .verification_tokens()
            .delete(verification.id)
            .await?;

        self.adapter
            .sessions()
            .delete_by_user_id(verification.user_id)
            .await?;

        if let Some(user) = self
            .adapter
            .users()
            .find_by_id(verification.user_id)
            .await?
        {
            self.send_webhook(
                WebhookEvent::PasswordChanged,
                WebhookData::User(UserWebhookData {
                    user_id: user.id,
                    email: user.email,
                    name: user.name,
                }),
            )
            .await;
        }

        Ok(())
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
        revoke_other_sessions: bool,
        current_session_id: Option<Uuid>,
    ) -> Result<()> {
        let accounts = self.adapter.accounts().find_by_user_id(user_id).await?;
        let _credential_account = accounts
            .iter()
            .find(|a| a.provider == "credential")
            .ok_or(TsaError::InvalidCredentials)?;

        let password_hash = self
            .get_password_hash(user_id)
            .await?
            .ok_or(TsaError::InvalidCredentials)?;

        if !Password::verify(current_password, &password_hash)? {
            return Err(TsaError::InvalidCredentials);
        }

        let new_hash = Password::hash(new_password)?;
        self.update_password_hash(user_id, &new_hash).await?;

        if revoke_other_sessions {
            if let Some(session_id) = current_session_id {
                self.revoke_other_sessions(user_id, session_id).await?;
            } else {
                self.revoke_all_sessions(user_id).await?;
            }
        }

        if let Some(user) = self.adapter.users().find_by_id(user_id).await? {
            self.send_webhook(
                WebhookEvent::PasswordChanged,
                WebhookData::User(UserWebhookData {
                    user_id: user.id,
                    email: user.email,
                    name: user.name,
                }),
            )
            .await;
        }

        Ok(())
    }

    pub(crate) async fn create_session_internal(
        &self,
        user: &User,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(Session, String)> {
        let (token, token_hash) =
            OpaqueToken::generate_with_hash(self.config.session_token_length)?;
        let now = Utc::now();

        let session = Session {
            id: Uuid::new_v4(),
            user_id: user.id,
            token_hash,
            expires_at: now + self.config.session_expiry,
            created_at: now,
            ip_address,
            user_agent,
        };

        let session = self.adapter.sessions().create(&session).await?;
        Ok((session, token))
    }

    async fn send_verification_email_internal(&self, user: &User) -> Result<()> {
        let (token, token_hash) = OpaqueToken::generate_with_hash(32)?;
        let now = Utc::now();

        let verification = VerificationToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            token_hash,
            token_type: TokenType::EmailVerification,
            expires_at: now + self.config.verification_token_expiry,
            created_at: now,
        };

        self.adapter
            .verification_tokens()
            .create(&verification)
            .await?;

        self.callbacks.send_verification_email(user, &token).await?;

        Ok(())
    }

    async fn create_credential_account(&self, user: &User, password_hash: &str) -> Result<Account> {
        let now = Utc::now();
        let account = Account {
            id: Uuid::new_v4(),
            user_id: user.id,
            provider: "credential".to_string(),
            provider_account_id: user.id.to_string(),
            access_token: Some(password_hash.to_string()),
            refresh_token: None,
            expires_at: None,
            created_at: now,
        };
        self.adapter.accounts().create(&account).await
    }

    async fn get_password_hash(&self, account_id: Uuid) -> Result<Option<String>> {
        let accounts = self.adapter.accounts().find_by_user_id(account_id).await?;
        Ok(accounts
            .into_iter()
            .find(|a| a.provider == "credential")
            .and_then(|a| a.access_token))
    }

    async fn update_password_hash(&self, user_id: Uuid, password_hash: &str) -> Result<()> {
        let accounts = self.adapter.accounts().find_by_user_id(user_id).await?;
        if let Some(credential_account) = accounts.into_iter().find(|a| a.provider == "credential")
        {
            let updated = Account {
                access_token: Some(password_hash.to_string()),
                ..credential_account
            };
            self.adapter.accounts().delete(updated.id).await?;
            self.adapter.accounts().create(&updated).await?;
        }
        Ok(())
    }

    #[cfg(feature = "totp")]
    pub async fn enable_2fa(&self, user_id: Uuid) -> Result<TwoFactorSetup> {
        if let Some(existing) = self.adapter.two_factor().find_by_user_id(user_id).await? {
            if existing.enabled {
                return Err(TsaError::TwoFactorAlreadyEnabled);
            }
            self.adapter.two_factor().delete(existing.id).await?;
        }

        let user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let totp_manager = TotpManager::new(&self.config.app_name);
        let secret = totp_manager.generate_secret();
        let otpauth_url = totp_manager.get_otpauth_url(&secret, &user.email)?;

        let backup_codes = BackupCodes::generate_default();
        let hashed_backup_codes = BackupCodes::hash_all(&backup_codes);

        let now = Utc::now();
        let two_factor = TwoFactor {
            id: Uuid::new_v4(),
            user_id,
            secret: secret.clone(),
            backup_codes: hashed_backup_codes,
            enabled: false,
            verified: false,
            created_at: now,
            updated_at: now,
        };

        self.adapter.two_factor().create(&two_factor).await?;

        Ok(TwoFactorSetup {
            secret,
            otpauth_url,
            backup_codes,
        })
    }

    #[cfg(feature = "totp")]
    pub async fn verify_2fa_setup(&self, user_id: Uuid, code: &str) -> Result<()> {
        let mut two_factor = self
            .adapter
            .two_factor()
            .find_by_user_id(user_id)
            .await?
            .ok_or(TsaError::TwoFactorNotEnabled)?;

        if two_factor.enabled {
            return Err(TsaError::TwoFactorAlreadyEnabled);
        }

        let user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let totp_manager = TotpManager::new(&self.config.app_name);
        if !totp_manager.verify(&two_factor.secret, code, &user.email)? {
            return Err(TsaError::InvalidTwoFactorCode);
        }

        two_factor.enabled = true;
        two_factor.verified = true;
        two_factor.updated_at = Utc::now();

        self.adapter.two_factor().update(&two_factor).await?;

        self.send_webhook(
            WebhookEvent::TwoFactorEnabled,
            WebhookData::User(UserWebhookData {
                user_id: user.id,
                email: user.email,
                name: user.name,
            }),
        )
        .await;

        Ok(())
    }

    #[cfg(feature = "totp")]
    pub async fn verify_2fa(&self, user_id: Uuid, code: &str) -> Result<TwoFactorMethod> {
        let mut two_factor = self
            .adapter
            .two_factor()
            .find_by_user_id(user_id)
            .await?
            .ok_or(TsaError::TwoFactorNotEnabled)?;

        if !two_factor.enabled {
            return Err(TsaError::TwoFactorNotEnabled);
        }

        let user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let totp_manager = TotpManager::new(&self.config.app_name);
        if totp_manager.verify(&two_factor.secret, code, &user.email)? {
            return Ok(TwoFactorMethod::Totp);
        }

        if let Some(index) = BackupCodes::verify(code, &two_factor.backup_codes) {
            two_factor.backup_codes.remove(index);
            two_factor.updated_at = Utc::now();
            self.adapter.two_factor().update(&two_factor).await?;
            return Ok(TwoFactorMethod::BackupCode);
        }

        Err(TsaError::InvalidTwoFactorCode)
    }

    #[cfg(feature = "totp")]
    pub async fn disable_2fa(&self, user_id: Uuid, code: &str) -> Result<()> {
        self.verify_2fa(user_id, code).await?;
        self.adapter.two_factor().delete_by_user_id(user_id).await?;

        if let Some(user) = self.adapter.users().find_by_id(user_id).await? {
            self.send_webhook(
                WebhookEvent::TwoFactorDisabled,
                WebhookData::User(UserWebhookData {
                    user_id: user.id,
                    email: user.email,
                    name: user.name,
                }),
            )
            .await;
        }

        Ok(())
    }

    #[cfg(feature = "totp")]
    pub async fn regenerate_backup_codes(&self, user_id: Uuid, code: &str) -> Result<Vec<String>> {
        self.verify_2fa(user_id, code).await?;

        let mut two_factor = self
            .adapter
            .two_factor()
            .find_by_user_id(user_id)
            .await?
            .ok_or(TsaError::TwoFactorNotEnabled)?;

        let backup_codes = BackupCodes::generate_default();
        let hashed_backup_codes = BackupCodes::hash_all(&backup_codes);

        two_factor.backup_codes = hashed_backup_codes;
        two_factor.updated_at = Utc::now();

        self.adapter.two_factor().update(&two_factor).await?;

        Ok(backup_codes)
    }

    #[cfg(feature = "totp")]
    pub async fn has_2fa_enabled(&self, user_id: Uuid) -> Result<bool> {
        let two_factor = self.adapter.two_factor().find_by_user_id(user_id).await?;
        Ok(two_factor.map(|t| t.enabled).unwrap_or(false))
    }

    #[cfg(feature = "totp")]
    pub async fn get_backup_codes_count(&self, user_id: Uuid) -> Result<usize> {
        let two_factor = self
            .adapter
            .two_factor()
            .find_by_user_id(user_id)
            .await?
            .ok_or(TsaError::TwoFactorNotEnabled)?;
        Ok(two_factor.backup_codes.len())
    }

    pub async fn send_magic_link(&self, email: &str) -> Result<()> {
        let user = match self.adapter.users().find_by_email(email).await? {
            Some(user) => user,
            None => return Ok(()),
        };

        let (token, token_hash) = OpaqueToken::generate_with_hash(32)?;
        let now = Utc::now();

        let verification = VerificationToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            token_hash,
            token_type: TokenType::MagicLink,
            expires_at: now + self.config.magic_link_expiry,
            created_at: now,
        };

        self.adapter
            .verification_tokens()
            .create(&verification)
            .await?;

        self.callbacks.send_magic_link_email(&user, &token).await?;

        self.send_webhook(
            WebhookEvent::MagicLinkSent,
            WebhookData::User(UserWebhookData {
                user_id: user.id,
                email: user.email,
                name: user.name,
            }),
        )
        .await;

        Ok(())
    }

    pub async fn verify_magic_link(
        &self,
        token: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, Session, String)> {
        let token_hash = OpaqueToken::hash(token);
        let verification = self
            .adapter
            .verification_tokens()
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(TsaError::InvalidToken)?;

        if verification.token_type != TokenType::MagicLink {
            return Err(TsaError::InvalidToken);
        }

        if verification.expires_at < Utc::now() {
            self.adapter
                .verification_tokens()
                .delete(verification.id)
                .await?;
            return Err(TsaError::TokenExpired);
        }

        let mut user = self
            .adapter
            .users()
            .find_by_id(verification.user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        if !user.email_verified {
            user.email_verified = true;
            user.updated_at = Utc::now();
            user = self.adapter.users().update(&user).await?;
        }

        self.adapter
            .verification_tokens()
            .delete(verification.id)
            .await?;

        let (session, session_token) = self
            .create_session_internal(&user, ip_address, user_agent)
            .await?;

        self.send_webhook(
            WebhookEvent::MagicLinkVerified,
            WebhookData::Auth(AuthWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
                failure_reason: None,
            }),
        )
        .await;

        self.send_webhook(
            WebhookEvent::SessionCreated,
            WebhookData::Session(SessionWebhookData {
                session_id: session.id,
                user_id: user.id,
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
            }),
        )
        .await;

        Ok((user, session, session_token))
    }

    pub async fn send_email_otp(&self, email: &str) -> Result<()> {
        let user = match self.adapter.users().find_by_email(email).await? {
            Some(user) => user,
            None => return Ok(()),
        };

        let code = Self::generate_otp_code();
        let code_hash = OpaqueToken::hash(&code);
        let now = Utc::now();

        let verification = VerificationToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            token_hash: code_hash,
            token_type: TokenType::EmailOtp,
            expires_at: now + self.config.otp_expiry,
            created_at: now,
        };

        self.adapter
            .verification_tokens()
            .create(&verification)
            .await?;

        self.callbacks.send_otp_email(&user, &code).await?;

        self.send_webhook(
            WebhookEvent::OtpSent,
            WebhookData::User(UserWebhookData {
                user_id: user.id,
                email: user.email,
                name: user.name,
            }),
        )
        .await;

        Ok(())
    }

    pub async fn verify_email_otp(
        &self,
        email: &str,
        code: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, Session, String)> {
        let user = self
            .adapter
            .users()
            .find_by_email(email)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let code_hash = OpaqueToken::hash(code);
        let verification = self
            .adapter
            .verification_tokens()
            .find_by_token_hash(&code_hash)
            .await?
            .ok_or(TsaError::InvalidToken)?;

        if verification.token_type != TokenType::EmailOtp {
            return Err(TsaError::InvalidToken);
        }

        if verification.user_id != user.id {
            return Err(TsaError::InvalidToken);
        }

        if verification.expires_at < Utc::now() {
            self.adapter
                .verification_tokens()
                .delete(verification.id)
                .await?;
            return Err(TsaError::TokenExpired);
        }

        let mut user = user;
        if !user.email_verified {
            user.email_verified = true;
            user.updated_at = Utc::now();
            user = self.adapter.users().update(&user).await?;
        }

        self.adapter
            .verification_tokens()
            .delete(verification.id)
            .await?;

        let (session, session_token) = self
            .create_session_internal(&user, ip_address, user_agent)
            .await?;

        self.send_webhook(
            WebhookEvent::OtpVerified,
            WebhookData::Auth(AuthWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
                failure_reason: None,
            }),
        )
        .await;

        self.send_webhook(
            WebhookEvent::SessionCreated,
            WebhookData::Session(SessionWebhookData {
                session_id: session.id,
                user_id: user.id,
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
            }),
        )
        .await;

        Ok((user, session, session_token))
    }

    fn generate_otp_code() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(100000..1000000);
        format!("{:06}", code)
    }

    pub async fn set_user_phone(&self, user_id: Uuid, phone: &str) -> Result<User> {
        let mut user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        user.phone = Some(phone.to_string());
        user.phone_verified = false;
        user.updated_at = Utc::now();

        self.adapter.users().update(&user).await
    }

    pub async fn send_phone_otp(&self, user_id: Uuid) -> Result<()> {
        let user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let phone = user.phone.as_ref().ok_or(TsaError::InvalidInput(
            "User has no phone number set".to_string(),
        ))?;

        let code = Self::generate_otp_code();
        let code_hash = OpaqueToken::hash(&code);
        let now = Utc::now();

        let verification = VerificationToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            token_hash: code_hash,
            token_type: TokenType::PhoneOtp,
            expires_at: now + self.config.otp_expiry,
            created_at: now,
        };

        self.adapter
            .verification_tokens()
            .create(&verification)
            .await?;

        self.callbacks.send_phone_otp(phone, &code).await?;

        self.send_webhook(
            WebhookEvent::OtpSent,
            WebhookData::User(UserWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                name: user.name.clone(),
            }),
        )
        .await;

        Ok(())
    }

    pub async fn verify_phone_otp(&self, user_id: Uuid, code: &str) -> Result<User> {
        let user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        let code_hash = OpaqueToken::hash(code);
        let verification = self
            .adapter
            .verification_tokens()
            .find_by_token_hash(&code_hash)
            .await?
            .ok_or(TsaError::InvalidToken)?;

        if verification.token_type != TokenType::PhoneOtp {
            return Err(TsaError::InvalidToken);
        }

        if verification.user_id != user.id {
            return Err(TsaError::InvalidToken);
        }

        if verification.expires_at < Utc::now() {
            self.adapter
                .verification_tokens()
                .delete(verification.id)
                .await?;
            return Err(TsaError::TokenExpired);
        }

        self.adapter
            .verification_tokens()
            .delete(verification.id)
            .await?;

        let mut user = user;
        user.phone_verified = true;
        user.updated_at = Utc::now();
        let user = self.adapter.users().update(&user).await?;

        self.send_webhook(
            WebhookEvent::PhoneVerified,
            WebhookData::User(UserWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                name: user.name.clone(),
            }),
        )
        .await;

        Ok(user)
    }

    pub async fn signin_with_phone_otp(&self, phone: &str) -> Result<()> {
        let users = self.find_users_by_phone(phone).await?;

        if users.is_empty() {
            return Ok(());
        }

        let user = &users[0];

        let code = Self::generate_otp_code();
        let code_hash = OpaqueToken::hash(&code);
        let now = Utc::now();

        let verification = VerificationToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            token_hash: code_hash,
            token_type: TokenType::PhoneOtp,
            expires_at: now + self.config.otp_expiry,
            created_at: now,
        };

        self.adapter
            .verification_tokens()
            .create(&verification)
            .await?;

        self.callbacks.send_phone_otp(phone, &code).await?;

        Ok(())
    }

    pub async fn verify_phone_signin(
        &self,
        phone: &str,
        code: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, Session, String)> {
        let users = self.find_users_by_phone(phone).await?;

        let user = users.into_iter().next().ok_or(TsaError::UserNotFound)?;

        let code_hash = OpaqueToken::hash(code);
        let verification = self
            .adapter
            .verification_tokens()
            .find_by_token_hash(&code_hash)
            .await?
            .ok_or(TsaError::InvalidToken)?;

        if verification.token_type != TokenType::PhoneOtp {
            return Err(TsaError::InvalidToken);
        }

        if verification.user_id != user.id {
            return Err(TsaError::InvalidToken);
        }

        if verification.expires_at < Utc::now() {
            self.adapter
                .verification_tokens()
                .delete(verification.id)
                .await?;
            return Err(TsaError::TokenExpired);
        }

        self.adapter
            .verification_tokens()
            .delete(verification.id)
            .await?;

        let mut user = user;
        if !user.phone_verified {
            user.phone_verified = true;
            user.updated_at = Utc::now();
            user = self.adapter.users().update(&user).await?;
        }

        let (session, session_token) = self
            .create_session_internal(&user, ip_address, user_agent)
            .await?;

        self.send_webhook(
            WebhookEvent::OtpVerified,
            WebhookData::Auth(AuthWebhookData {
                user_id: user.id,
                email: user.email.clone(),
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
                failure_reason: None,
            }),
        )
        .await;

        self.send_webhook(
            WebhookEvent::SessionCreated,
            WebhookData::Session(SessionWebhookData {
                session_id: session.id,
                user_id: user.id,
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
            }),
        )
        .await;

        Ok((user, session, session_token))
    }

    async fn find_users_by_phone(&self, phone: &str) -> Result<Vec<User>> {
        match self.adapter.users().find_by_phone(phone).await? {
            Some(user) => Ok(vec![user]),
            None => Ok(Vec::new()),
        }
    }

    pub async fn create_organization(
        &self,
        user_id: Uuid,
        name: &str,
        slug: &str,
    ) -> Result<(Organization, OrganizationMember)> {
        if self
            .adapter
            .organizations()
            .find_by_slug(slug)
            .await?
            .is_some()
        {
            return Err(TsaError::OrganizationAlreadyExists);
        }

        let now = Utc::now();
        let organization = Organization {
            id: Uuid::new_v4(),
            name: name.to_string(),
            slug: slug.to_lowercase(),
            logo: None,
            metadata: None,
            created_at: now,
            updated_at: now,
        };

        let organization = self.adapter.organizations().create(&organization).await?;

        let member = OrganizationMember {
            id: Uuid::new_v4(),
            organization_id: organization.id,
            user_id,
            role: OrganizationRole::Owner,
            created_at: now,
            updated_at: now,
        };

        let member = self.adapter.organization_members().create(&member).await?;

        self.send_webhook(
            WebhookEvent::OrganizationCreated,
            WebhookData::Organization(OrganizationWebhookData {
                organization_id: organization.id,
                name: organization.name.clone(),
                slug: organization.slug.clone(),
            }),
        )
        .await;

        self.send_webhook(
            WebhookEvent::MemberAdded,
            WebhookData::Member(MemberWebhookData {
                organization_id: organization.id,
                user_id,
                role: member.role.to_string(),
                previous_role: None,
            }),
        )
        .await;

        Ok((organization, member))
    }

    pub async fn get_organization(&self, organization_id: Uuid) -> Result<Organization> {
        self.adapter
            .organizations()
            .find_by_id(organization_id)
            .await?
            .ok_or(TsaError::OrganizationNotFound)
    }

    pub async fn get_organization_by_slug(&self, slug: &str) -> Result<Organization> {
        self.adapter
            .organizations()
            .find_by_slug(slug)
            .await?
            .ok_or(TsaError::OrganizationNotFound)
    }

    pub async fn update_organization(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        name: Option<String>,
        logo: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Organization> {
        self.require_org_role(
            user_id,
            organization_id,
            &[OrganizationRole::Owner, OrganizationRole::Admin],
        )
        .await?;

        let mut organization = self.get_organization(organization_id).await?;

        if let Some(name) = name {
            organization.name = name;
        }
        if let Some(logo) = logo {
            organization.logo = Some(logo);
        }
        if let Some(metadata) = metadata {
            organization.metadata = Some(metadata);
        }
        organization.updated_at = Utc::now();

        let organization = self.adapter.organizations().update(&organization).await?;

        self.send_webhook(
            WebhookEvent::OrganizationUpdated,
            WebhookData::Organization(OrganizationWebhookData {
                organization_id: organization.id,
                name: organization.name.clone(),
                slug: organization.slug.clone(),
            }),
        )
        .await;

        Ok(organization)
    }

    pub async fn delete_organization(&self, user_id: Uuid, organization_id: Uuid) -> Result<()> {
        self.require_org_role(user_id, organization_id, &[OrganizationRole::Owner])
            .await?;

        let organization = self.get_organization(organization_id).await?;

        self.adapter
            .organization_members()
            .delete_by_organization(organization_id)
            .await?;
        self.adapter.organizations().delete(organization_id).await?;

        self.send_webhook(
            WebhookEvent::OrganizationDeleted,
            WebhookData::Organization(OrganizationWebhookData {
                organization_id: organization.id,
                name: organization.name,
                slug: organization.slug,
            }),
        )
        .await;

        Ok(())
    }

    pub async fn get_user_organizations(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(Organization, OrganizationRole)>> {
        let memberships = self
            .adapter
            .organization_members()
            .find_by_user(user_id)
            .await?;
        let mut result = Vec::new();

        for membership in memberships {
            if let Some(org) = self
                .adapter
                .organizations()
                .find_by_id(membership.organization_id)
                .await?
            {
                result.push((org, membership.role));
            }
        }

        Ok(result)
    }

    pub async fn get_organization_members(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<(User, OrganizationMember)>> {
        let members = self
            .adapter
            .organization_members()
            .find_by_organization(organization_id)
            .await?;

        let mut result = Vec::new();
        for member in members {
            if let Some(user) = self.adapter.users().find_by_id(member.user_id).await? {
                result.push((user, member));
            }
        }

        Ok(result)
    }

    pub async fn add_organization_member(
        &self,
        inviter_id: Uuid,
        organization_id: Uuid,
        user_id: Uuid,
        role: OrganizationRole,
    ) -> Result<OrganizationMember> {
        self.require_org_role(
            inviter_id,
            organization_id,
            &[OrganizationRole::Owner, OrganizationRole::Admin],
        )
        .await?;

        if role == OrganizationRole::Owner {
            let inviter_member = self.get_member(inviter_id, organization_id).await?;
            if inviter_member.role != OrganizationRole::Owner {
                return Err(TsaError::InsufficientPermissions);
            }
        }

        let now = Utc::now();
        let member = OrganizationMember {
            id: Uuid::new_v4(),
            organization_id,
            user_id,
            role,
            created_at: now,
            updated_at: now,
        };

        let member = self.adapter.organization_members().create(&member).await?;

        self.send_webhook(
            WebhookEvent::MemberAdded,
            WebhookData::Member(MemberWebhookData {
                organization_id,
                user_id,
                role: member.role.to_string(),
                previous_role: None,
            }),
        )
        .await;

        Ok(member)
    }

    pub async fn update_member_role(
        &self,
        updater_id: Uuid,
        organization_id: Uuid,
        member_user_id: Uuid,
        new_role: OrganizationRole,
    ) -> Result<OrganizationMember> {
        self.require_org_role(
            updater_id,
            organization_id,
            &[OrganizationRole::Owner, OrganizationRole::Admin],
        )
        .await?;

        let updater_member = self.get_member(updater_id, organization_id).await?;
        let mut member = self.get_member(member_user_id, organization_id).await?;

        if member.role == OrganizationRole::Owner && updater_member.role != OrganizationRole::Owner
        {
            return Err(TsaError::InsufficientPermissions);
        }

        if new_role == OrganizationRole::Owner && updater_member.role != OrganizationRole::Owner {
            return Err(TsaError::InsufficientPermissions);
        }

        if member.role == OrganizationRole::Owner && new_role != OrganizationRole::Owner {
            let owners = self.count_owners(organization_id).await?;
            if owners <= 1 {
                return Err(TsaError::CannotRemoveLastOwner);
            }
        }

        let previous_role = member.role.to_string();
        member.role = new_role;
        member.updated_at = Utc::now();

        let member = self.adapter.organization_members().update(&member).await?;

        self.send_webhook(
            WebhookEvent::MemberRoleChanged,
            WebhookData::Member(MemberWebhookData {
                organization_id,
                user_id: member_user_id,
                role: member.role.to_string(),
                previous_role: Some(previous_role),
            }),
        )
        .await;

        Ok(member)
    }

    pub async fn remove_organization_member(
        &self,
        remover_id: Uuid,
        organization_id: Uuid,
        member_user_id: Uuid,
    ) -> Result<()> {
        let member = self.get_member(member_user_id, organization_id).await?;

        if remover_id == member_user_id {
            if member.role == OrganizationRole::Owner {
                let owners = self.count_owners(organization_id).await?;
                if owners <= 1 {
                    return Err(TsaError::CannotRemoveLastOwner);
                }
            }
        } else {
            self.require_org_role(
                remover_id,
                organization_id,
                &[OrganizationRole::Owner, OrganizationRole::Admin],
            )
            .await?;

            let remover_member = self.get_member(remover_id, organization_id).await?;
            if member.role == OrganizationRole::Owner
                && remover_member.role != OrganizationRole::Owner
            {
                return Err(TsaError::InsufficientPermissions);
            }
        }

        self.adapter
            .organization_members()
            .delete(member.id)
            .await?;

        self.send_webhook(
            WebhookEvent::MemberRemoved,
            WebhookData::Member(MemberWebhookData {
                organization_id,
                user_id: member_user_id,
                role: member.role.to_string(),
                previous_role: None,
            }),
        )
        .await;

        Ok(())
    }

    pub async fn invite_to_organization(
        &self,
        inviter_id: Uuid,
        organization_id: Uuid,
        email: &str,
        role: OrganizationRole,
    ) -> Result<String> {
        self.require_org_role(
            inviter_id,
            organization_id,
            &[OrganizationRole::Owner, OrganizationRole::Admin],
        )
        .await?;

        if role == OrganizationRole::Owner {
            let inviter_member = self.get_member(inviter_id, organization_id).await?;
            if inviter_member.role != OrganizationRole::Owner {
                return Err(TsaError::InsufficientPermissions);
            }
        }

        if let Some(existing) = self
            .adapter
            .organization_invitations()
            .find_pending_by_org_and_email(organization_id, email)
            .await?
        {
            self.adapter
                .organization_invitations()
                .delete(existing.id)
                .await?;
        }

        let (token, token_hash) = OpaqueToken::generate_with_hash(32)?;
        let now = Utc::now();

        let invitation = OrganizationInvitation {
            id: Uuid::new_v4(),
            organization_id,
            email: email.to_string(),
            role,
            token_hash,
            invited_by: inviter_id,
            status: InvitationStatus::Pending,
            expires_at: now + chrono::Duration::days(7),
            created_at: now,
        };

        self.adapter
            .organization_invitations()
            .create(&invitation)
            .await?;

        self.send_webhook(
            WebhookEvent::InvitationSent,
            WebhookData::Invitation(InvitationWebhookData {
                invitation_id: invitation.id,
                organization_id,
                email: email.to_string(),
                role: invitation.role.to_string(),
                inviter_id: Some(inviter_id),
            }),
        )
        .await;

        Ok(token)
    }

    pub async fn accept_invitation(
        &self,
        user_id: Uuid,
        token: &str,
    ) -> Result<OrganizationMember> {
        let token_hash = OpaqueToken::hash(token);
        let invitation = self
            .adapter
            .organization_invitations()
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(TsaError::InvitationNotFound)?;

        if invitation.status != InvitationStatus::Pending {
            return Err(TsaError::InvitationAlreadyUsed);
        }

        if invitation.expires_at < Utc::now() {
            self.adapter
                .organization_invitations()
                .update_status(invitation.id, InvitationStatus::Expired)
                .await?;
            return Err(TsaError::InvitationExpired);
        }

        let user = self
            .adapter
            .users()
            .find_by_id(user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        if user.email.to_lowercase() != invitation.email.to_lowercase() {
            return Err(TsaError::InvitationNotFound);
        }

        let now = Utc::now();
        let member = OrganizationMember {
            id: Uuid::new_v4(),
            organization_id: invitation.organization_id,
            user_id,
            role: invitation.role.clone(),
            created_at: now,
            updated_at: now,
        };

        let member = self.adapter.organization_members().create(&member).await?;

        self.adapter
            .organization_invitations()
            .update_status(invitation.id, InvitationStatus::Accepted)
            .await?;

        self.send_webhook(
            WebhookEvent::InvitationAccepted,
            WebhookData::Invitation(InvitationWebhookData {
                invitation_id: invitation.id,
                organization_id: invitation.organization_id,
                email: invitation.email,
                role: member.role.to_string(),
                inviter_id: Some(invitation.invited_by),
            }),
        )
        .await;

        self.send_webhook(
            WebhookEvent::MemberAdded,
            WebhookData::Member(MemberWebhookData {
                organization_id: member.organization_id,
                user_id,
                role: member.role.to_string(),
                previous_role: None,
            }),
        )
        .await;

        Ok(member)
    }

    pub async fn revoke_invitation(&self, user_id: Uuid, invitation_id: Uuid) -> Result<()> {
        let invitation = self
            .adapter
            .organization_invitations()
            .find_by_id(invitation_id)
            .await?
            .ok_or(TsaError::InvitationNotFound)?;

        self.require_org_role(
            user_id,
            invitation.organization_id,
            &[OrganizationRole::Owner, OrganizationRole::Admin],
        )
        .await?;

        self.adapter
            .organization_invitations()
            .update_status(invitation_id, InvitationStatus::Revoked)
            .await?;

        self.send_webhook(
            WebhookEvent::InvitationRevoked,
            WebhookData::Invitation(InvitationWebhookData {
                invitation_id,
                organization_id: invitation.organization_id,
                email: invitation.email,
                role: invitation.role.to_string(),
                inviter_id: Some(invitation.invited_by),
            }),
        )
        .await;

        Ok(())
    }

    pub async fn get_organization_invitations(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OrganizationInvitation>> {
        self.adapter
            .organization_invitations()
            .find_by_organization(organization_id)
            .await
    }

    async fn get_member(&self, user_id: Uuid, organization_id: Uuid) -> Result<OrganizationMember> {
        self.adapter
            .organization_members()
            .find_by_org_and_user(organization_id, user_id)
            .await?
            .ok_or(TsaError::NotOrganizationMember)
    }

    async fn require_org_role(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        allowed_roles: &[OrganizationRole],
    ) -> Result<()> {
        let member = self.get_member(user_id, organization_id).await?;
        if !allowed_roles.contains(&member.role) {
            return Err(TsaError::InsufficientPermissions);
        }
        Ok(())
    }

    async fn count_owners(&self, organization_id: Uuid) -> Result<usize> {
        let members = self
            .adapter
            .organization_members()
            .find_by_organization(organization_id)
            .await?;
        Ok(members
            .iter()
            .filter(|m| m.role == OrganizationRole::Owner)
            .count())
    }

    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        name: &str,
        scopes: Vec<String>,
        organization_id: Option<Uuid>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<(ApiKey, String)> {
        if let Some(org_id) = organization_id {
            self.require_org_role(
                user_id,
                org_id,
                &[OrganizationRole::Owner, OrganizationRole::Admin],
            )
            .await?;
        }

        let (key, key_hash) = OpaqueToken::generate_with_hash(32)?;
        let prefix = format!("tsa_{}", &key[..8]);

        let now = Utc::now();
        let api_key = ApiKey {
            id: Uuid::new_v4(),
            user_id,
            organization_id,
            name: name.to_string(),
            key_hash,
            prefix: prefix.clone(),
            scopes,
            expires_at,
            last_used_at: None,
            created_at: now,
        };

        let api_key = self.adapter.api_keys().create(&api_key).await?;
        let full_key = format!("{}_{}", prefix, key);

        self.send_webhook(
            WebhookEvent::ApiKeyCreated,
            WebhookData::ApiKey(ApiKeyWebhookData {
                api_key_id: api_key.id,
                user_id,
                name: api_key.name.clone(),
                prefix: Some(api_key.prefix.clone()),
            }),
        )
        .await;

        Ok((api_key, full_key))
    }

    pub async fn validate_api_key(&self, key: &str) -> Result<(ApiKey, User)> {
        if !key.starts_with("tsa_") {
            return Err(TsaError::InvalidApiKey);
        }

        let after_prefix = &key[4..];
        let parts: Vec<&str> = after_prefix.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(TsaError::InvalidApiKey);
        }

        let key_part = parts[1];
        let key_hash = OpaqueToken::hash(key_part);

        let api_key = self
            .adapter
            .api_keys()
            .find_by_key_hash(&key_hash)
            .await?
            .ok_or(TsaError::InvalidApiKey)?;

        if let Some(expires_at) = api_key.expires_at {
            if expires_at < Utc::now() {
                return Err(TsaError::InvalidApiKey);
            }
        }

        let user = self
            .adapter
            .users()
            .find_by_id(api_key.user_id)
            .await?
            .ok_or(TsaError::UserNotFound)?;

        self.adapter.api_keys().update_last_used(api_key.id).await?;

        Ok((api_key, user))
    }

    pub async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        self.adapter.api_keys().find_by_user(user_id).await
    }

    pub async fn list_organization_api_keys(&self, organization_id: Uuid) -> Result<Vec<ApiKey>> {
        self.adapter
            .api_keys()
            .find_by_organization(organization_id)
            .await
    }

    pub async fn delete_api_key(&self, user_id: Uuid, api_key_id: Uuid) -> Result<()> {
        let api_key = self
            .adapter
            .api_keys()
            .find_by_id(api_key_id)
            .await?
            .ok_or(TsaError::ApiKeyNotFound)?;

        if api_key.user_id != user_id {
            if let Some(org_id) = api_key.organization_id {
                self.require_org_role(
                    user_id,
                    org_id,
                    &[OrganizationRole::Owner, OrganizationRole::Admin],
                )
                .await?;
            } else {
                return Err(TsaError::InsufficientPermissions);
            }
        }

        self.adapter.api_keys().delete(api_key_id).await?;

        self.send_webhook(
            WebhookEvent::ApiKeyRevoked,
            WebhookData::ApiKey(ApiKeyWebhookData {
                api_key_id: api_key.id,
                user_id: api_key.user_id,
                name: api_key.name,
                prefix: Some(api_key.prefix),
            }),
        )
        .await;

        Ok(())
    }

    pub async fn update_api_key(
        &self,
        user_id: Uuid,
        api_key_id: Uuid,
        name: Option<String>,
        scopes: Option<Vec<String>>,
    ) -> Result<ApiKey> {
        let mut api_key = self
            .adapter
            .api_keys()
            .find_by_id(api_key_id)
            .await?
            .ok_or(TsaError::ApiKeyNotFound)?;

        if api_key.user_id != user_id {
            if let Some(org_id) = api_key.organization_id {
                self.require_org_role(
                    user_id,
                    org_id,
                    &[OrganizationRole::Owner, OrganizationRole::Admin],
                )
                .await?;
            } else {
                return Err(TsaError::InsufficientPermissions);
            }
        }

        if let Some(name) = name {
            api_key.name = name;
        }
        if let Some(scopes) = scopes {
            api_key.scopes = scopes;
        }

        self.adapter.api_keys().update(&api_key).await
    }
}
