use chrono::Utc;
use tsa_core::{
    Account, AccountRepository, Adapter, Result, Session, TsaError, User, UserRepository,
};
use uuid::Uuid;

use crate::{Auth, AuthCallbacks};
use tsa_oauth::{OAuthRegistry, OAuthTokens, OAuthUserInfo, ProviderConfig};

pub struct OAuthResult {
    pub user: User,
    pub session: Session,
    pub token: String,
    pub is_new_user: bool,
}

impl<A: Adapter, C: AuthCallbacks> Auth<A, C> {
    pub fn with_oauth(self, secret: impl Into<String>) -> AuthWithOAuth<A, C> {
        AuthWithOAuth {
            auth: self,
            oauth: OAuthRegistry::new(secret),
        }
    }
}

pub struct AuthWithOAuth<A: Adapter, C: AuthCallbacks> {
    auth: Auth<A, C>,
    oauth: OAuthRegistry,
}

impl<A: Adapter, C: AuthCallbacks> AuthWithOAuth<A, C> {
    pub fn provider(
        mut self,
        name: &str,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> Result<Self> {
        let config = ProviderConfig {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            redirect_url: redirect_url.into(),
            scopes: None,
        };

        match name {
            "google" => self.oauth.register_google(config)?,
            "github" => self.oauth.register_github(config)?,
            "discord" => self.oauth.register_discord(config)?,
            "apple" => self.oauth.register_apple(config)?,
            "microsoft" => self.oauth.register_microsoft(config)?,
            "gitlab" => self.oauth.register_gitlab(config)?,
            "twitter" => self.oauth.register_twitter(config)?,
            "facebook" => self.oauth.register_facebook(config)?,
            "linkedin" => self.oauth.register_linkedin(config)?,
            "spotify" => self.oauth.register_spotify(config)?,
            "slack" => self.oauth.register_slack(config)?,
            "twitch" => self.oauth.register_twitch(config)?,
            _ => {
                return Err(TsaError::Configuration(format!(
                    "Unknown provider: {}",
                    name
                )))
            }
        }

        Ok(self)
    }

    pub fn oauth_url(&self, provider: &str) -> Result<(String, String)> {
        self.oauth.authorization_url(provider)
    }

    pub async fn oauth_callback(
        &self,
        provider: &str,
        code: &str,
        state: &str,
    ) -> Result<OAuthResult> {
        self.oauth_callback_with_context(provider, code, state, None, None)
            .await
    }

    pub async fn oauth_callback_with_context(
        &self,
        provider: &str,
        code: &str,
        state: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<OAuthResult> {
        let (tokens, user_info) = self.oauth.exchange_code(provider, code, state).await?;

        self.signin_or_signup(provider, user_info, tokens, ip_address, user_agent)
            .await
    }

    pub async fn oauth_link(
        &self,
        user_id: Uuid,
        provider: &str,
        code: &str,
        state: &str,
    ) -> Result<Account> {
        let (tokens, user_info) = self.oauth.exchange_code(provider, code, state).await?;

        let existing = self
            .auth
            .adapter
            .accounts()
            .find_by_provider(provider, &user_info.provider_user_id)
            .await?;

        if existing.is_some() {
            return Err(TsaError::AccountAlreadyLinked);
        }

        let now = Utc::now();
        let account = Account {
            id: Uuid::new_v4(),
            user_id,
            provider: provider.to_string(),
            provider_account_id: user_info.provider_user_id,
            access_token: Some(tokens.access_token),
            refresh_token: tokens.refresh_token,
            expires_at: tokens
                .expires_in
                .map(|d| now + chrono::Duration::from_std(d).unwrap_or_default()),
            created_at: now,
        };

        self.auth.adapter.accounts().create(&account).await
    }

    pub async fn oauth_unlink(&self, user_id: Uuid, provider: &str) -> Result<()> {
        let accounts = self
            .auth
            .adapter
            .accounts()
            .find_by_user_id(user_id)
            .await?;

        let has_credential = accounts.iter().any(|a| a.provider == "credential");
        let oauth_count = accounts
            .iter()
            .filter(|a| a.provider != "credential")
            .count();

        if !has_credential && oauth_count <= 1 {
            return Err(TsaError::CannotUnlinkLastAccount);
        }

        if let Some(account) = accounts.into_iter().find(|a| a.provider == provider) {
            self.auth.adapter.accounts().delete(account.id).await?;
        }

        Ok(())
    }

    async fn signin_or_signup(
        &self,
        provider: &str,
        user_info: OAuthUserInfo,
        tokens: OAuthTokens,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<OAuthResult> {
        let existing_account = self
            .auth
            .adapter
            .accounts()
            .find_by_provider(provider, &user_info.provider_user_id)
            .await?;

        if let Some(account) = existing_account {
            let user = self
                .auth
                .adapter
                .users()
                .find_by_id(account.user_id)
                .await?
                .ok_or(TsaError::UserNotFound)?;

            self.update_tokens(account.id, &tokens).await?;

            let (session, token) = self.create_session(&user, ip_address, user_agent).await?;

            return Ok(OAuthResult {
                user,
                session,
                token,
                is_new_user: false,
            });
        }

        if let Some(ref email) = user_info.email {
            if let Some(existing_user) = self.auth.adapter.users().find_by_email(email).await? {
                self.link_account(&existing_user, provider, &user_info, &tokens)
                    .await?;

                if !existing_user.email_verified && user_info.email_verified.unwrap_or(false) {
                    let mut updated = existing_user.clone();
                    updated.email_verified = true;
                    updated.updated_at = Utc::now();
                    self.auth.adapter.users().update(&updated).await?;
                }

                let (session, token) = self
                    .create_session(&existing_user, ip_address, user_agent)
                    .await?;

                return Ok(OAuthResult {
                    user: existing_user,
                    session,
                    token,
                    is_new_user: false,
                });
            }
        }

        let now = Utc::now();
        let user = User {
            id: Uuid::new_v4(),
            email: user_info
                .email
                .clone()
                .unwrap_or_else(|| format!("{}@{}.oauth", user_info.provider_user_id, provider)),
            email_verified: user_info.email_verified.unwrap_or(false),
            phone: None,
            phone_verified: false,
            name: user_info.name.clone(),
            image: user_info.image.clone(),
            created_at: now,
            updated_at: now,
        };

        let user = self.auth.adapter.users().create(&user).await?;

        self.link_account(&user, provider, &user_info, &tokens)
            .await?;

        self.auth.callbacks.on_user_created(&user).await?;

        let (session, token) = self.create_session(&user, ip_address, user_agent).await?;

        Ok(OAuthResult {
            user,
            session,
            token,
            is_new_user: true,
        })
    }

    async fn link_account(
        &self,
        user: &User,
        provider: &str,
        user_info: &OAuthUserInfo,
        tokens: &OAuthTokens,
    ) -> Result<Account> {
        let now = Utc::now();
        let account = Account {
            id: Uuid::new_v4(),
            user_id: user.id,
            provider: provider.to_string(),
            provider_account_id: user_info.provider_user_id.clone(),
            access_token: Some(tokens.access_token.clone()),
            refresh_token: tokens.refresh_token.clone(),
            expires_at: tokens
                .expires_in
                .map(|d| now + chrono::Duration::from_std(d).unwrap_or_default()),
            created_at: now,
        };

        self.auth.adapter.accounts().create(&account).await
    }

    async fn create_session(
        &self,
        user: &User,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(Session, String)> {
        use tsa_core::SessionRepository;
        use tsa_token::OpaqueToken;

        let (token, token_hash) =
            OpaqueToken::generate_with_hash(self.auth.config.session_token_length)?;
        let now = Utc::now();

        let session = Session {
            id: Uuid::new_v4(),
            user_id: user.id,
            token_hash,
            expires_at: now + self.auth.config.session_expiry,
            created_at: now,
            ip_address,
            user_agent,
        };

        let session = self.auth.adapter.sessions().create(&session).await?;
        Ok((session, token))
    }

    async fn update_tokens(&self, account_id: Uuid, tokens: &OAuthTokens) -> Result<()> {
        let accounts = self
            .auth
            .adapter
            .accounts()
            .find_by_user_id(account_id)
            .await?;

        if let Some(mut account) = accounts.into_iter().find(|a| a.id == account_id) {
            account.access_token = Some(tokens.access_token.clone());
            if let Some(ref rt) = tokens.refresh_token {
                account.refresh_token = Some(rt.clone());
            }
            if let Some(exp) = tokens.expires_in {
                account.expires_at =
                    Some(Utc::now() + chrono::Duration::from_std(exp).unwrap_or_default());
            }
            self.auth.adapter.accounts().delete(account.id).await?;
            self.auth.adapter.accounts().create(&account).await?;
        }
        Ok(())
    }

    // Delegate to inner Auth
    pub async fn signup(
        &self,
        email: &str,
        password: &str,
        name: Option<String>,
    ) -> Result<(User, Session, String)> {
        self.auth.signup(email, password, name).await
    }

    pub async fn signin(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, Session, String)> {
        self.auth
            .signin(email, password, ip_address, user_agent)
            .await
    }

    pub async fn signout(&self, session_token: &str) -> Result<()> {
        self.auth.signout(session_token).await
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<(User, Session)> {
        self.auth.validate_session(session_token).await
    }

    pub fn providers(&self) -> Vec<&str> {
        self.oauth.providers()
    }
}
