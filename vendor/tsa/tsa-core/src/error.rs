use thiserror::Error;

#[derive(Debug, Error)]
pub enum TsaError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Session expired")]
    SessionExpired,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Account already linked")]
    AccountAlreadyLinked,

    #[error("Cannot unlink last account")]
    CannotUnlinkLastAccount,

    #[error("Two-factor authentication not enabled")]
    TwoFactorNotEnabled,

    #[error("Two-factor authentication already enabled")]
    TwoFactorAlreadyEnabled,

    #[error("Invalid two-factor authentication code")]
    InvalidTwoFactorCode,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Organization not found")]
    OrganizationNotFound,

    #[error("Organization already exists")]
    OrganizationAlreadyExists,

    #[error("Already a member of this organization")]
    AlreadyOrganizationMember,

    #[error("Not a member of this organization")]
    NotOrganizationMember,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Cannot remove last owner")]
    CannotRemoveLastOwner,

    #[error("Invitation not found")]
    InvitationNotFound,

    #[error("Invitation expired")]
    InvitationExpired,

    #[error("Invitation already used")]
    InvitationAlreadyUsed,

    #[error("API key not found")]
    ApiKeyNotFound,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Passkey not found")]
    PasskeyNotFound,

    #[error("Passkey already registered")]
    PasskeyAlreadyRegistered,

    #[error("Passkey challenge not found")]
    PasskeyChallengeNotFound,

    #[error("Passkey challenge expired")]
    PasskeyChallengeExpired,

    #[error("Passkey verification failed")]
    PasskeyVerificationFailed,

    #[error("Account is locked")]
    AccountLocked,

    #[error("Password policy violation: {0}")]
    PasswordPolicyViolation(String),

    #[error("Password was recently used")]
    PasswordRecentlyUsed,

    #[error("IP address blocked")]
    IpBlocked,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Password hash error: {0}")]
    PasswordHash(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, TsaError>;
