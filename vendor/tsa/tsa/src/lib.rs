#![allow(clippy::too_many_arguments)]
#![allow(unexpected_cfgs)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod auth;
mod callbacks;
mod config;
mod password;
pub mod rate_limit;
pub mod security;
pub mod two_factor;
pub mod webhook;

#[cfg(feature = "passkey")]
pub mod passkey;

#[cfg(feature = "axum")]
pub mod axum_integration;

#[cfg(feature = "oauth")]
mod oauth_auth;

#[cfg(feature = "oauth")]
pub use tsa_oauth as oauth;

#[cfg(feature = "enterprise")]
pub use tsa_enterprise as enterprise;

pub use auth::Auth;
pub use callbacks::{AuthCallbacks, NoopCallbacks};
pub use config::AuthConfig;
pub use password::Password;
pub use rate_limit::{
    InMemoryRateLimiter, RateLimitAction, RateLimitConfig, RateLimitResult, RateLimiter,
};
pub use two_factor::{BackupCodes, TwoFactorMethod, TwoFactorSetup};

#[cfg(feature = "totp")]
pub use two_factor::TotpManager;

#[cfg(feature = "passkey")]
pub use passkey::{
    PasskeyAuthenticationStart, PasskeyConfig, PasskeyManager, PasskeyRegistrationStart,
};

#[cfg(feature = "oauth")]
pub use oauth_auth::{AuthWithOAuth, OAuthResult};

pub use tsa_core as core;
pub use tsa_session as session;
pub use tsa_token as token;

pub use tsa_core::{
    Account, AccountRepository, Adapter, ApprovalDecision, ApprovalRequest,
    ApprovalRequestRepository, ApprovalResponse, ApprovalResponseRepository, ApprovalStatus,
    ApprovalToken, ApprovalTokenRepository, Passkey, PasskeyChallenge, PasskeyChallengeRepository,
    PasskeyChallengeType, PasskeyRepository, Result, Session, SessionRepository, TokenType,
    TsaError, TwoFactor, TwoFactorRepository, User, UserRepository, VerificationToken,
    VerificationTokenRepository, WebhookData, WebhookEvent, WebhookPayload,
};

pub use webhook::{
    verify_webhook_signature, AsyncWebhookSender, HttpWebhookSender, NoopWebhookSender,
    WebhookConfig, WebhookSender,
};

pub use security::{SecurityConfig, SecurityManager};

pub mod adapter {
    pub use tsa_adapter::*;

    #[cfg(feature = "adapter-seaorm")]
    pub use tsa_adapter_seaorm as seaorm;

    #[cfg(feature = "adapter-mongodb")]
    pub use tsa_adapter_mongodb as mongodb;

    #[cfg(feature = "adapter-dynamodb")]
    pub use tsa_adapter_dynamodb as dynamodb;

    #[cfg(feature = "adapter-firestore")]
    pub use tsa_adapter_firestore as firestore;

    #[cfg(feature = "adapter-bigtable")]
    pub use tsa_adapter_bigtable as bigtable;

    #[cfg(feature = "adapter-redis")]
    pub use tsa_adapter_redis as redis;
}
