mod memory;

pub use memory::InMemoryRateLimiter;

use async_trait::async_trait;
use std::time::Duration;
use tsa_core::Result;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 10,
            window: Duration::from_secs(60),
        }
    }
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }

    pub fn per_minute(max_requests: u32) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(60),
        }
    }

    pub fn per_hour(max_requests: u32) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(3600),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_at: std::time::Instant,
}

#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check(&self, key: &str, config: &RateLimitConfig) -> Result<RateLimitResult>;
    async fn reset(&self, key: &str) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitAction {
    SignIn,
    SignUp,
    PasswordReset,
    EmailVerification,
    TwoFactorVerify,
}

impl RateLimitAction {
    pub fn default_config(&self) -> RateLimitConfig {
        match self {
            RateLimitAction::SignIn => RateLimitConfig::new(5, Duration::from_secs(60)),
            RateLimitAction::SignUp => RateLimitConfig::new(3, Duration::from_secs(300)),
            RateLimitAction::PasswordReset => RateLimitConfig::new(3, Duration::from_secs(3600)),
            RateLimitAction::EmailVerification => RateLimitConfig::new(5, Duration::from_secs(300)),
            RateLimitAction::TwoFactorVerify => RateLimitConfig::new(5, Duration::from_secs(300)),
        }
    }

    pub fn key_prefix(&self) -> &'static str {
        match self {
            RateLimitAction::SignIn => "signin",
            RateLimitAction::SignUp => "signup",
            RateLimitAction::PasswordReset => "pwreset",
            RateLimitAction::EmailVerification => "emailver",
            RateLimitAction::TwoFactorVerify => "2fa",
        }
    }
}
