use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    UserCreated,
    UserUpdated,
    UserDeleted,
    EmailVerified,

    SigninSuccess,
    SigninFailed,
    SignoutSuccess,

    PasswordChanged,
    PasswordResetRequested,

    SessionCreated,
    SessionRevoked,

    TwoFactorEnabled,
    TwoFactorDisabled,

    OrganizationCreated,
    OrganizationUpdated,
    OrganizationDeleted,

    MemberAdded,
    MemberRemoved,
    MemberRoleChanged,

    InvitationSent,
    InvitationAccepted,
    InvitationRevoked,

    ApiKeyCreated,
    ApiKeyRevoked,

    MagicLinkSent,
    MagicLinkVerified,

    OtpSent,
    OtpVerified,

    PhoneVerified,
}

impl WebhookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserCreated => "user.created",
            Self::UserUpdated => "user.updated",
            Self::UserDeleted => "user.deleted",
            Self::EmailVerified => "user.email_verified",

            Self::SigninSuccess => "auth.signin_success",
            Self::SigninFailed => "auth.signin_failed",
            Self::SignoutSuccess => "auth.signout",

            Self::PasswordChanged => "auth.password_changed",
            Self::PasswordResetRequested => "auth.password_reset_requested",

            Self::SessionCreated => "session.created",
            Self::SessionRevoked => "session.revoked",

            Self::TwoFactorEnabled => "2fa.enabled",
            Self::TwoFactorDisabled => "2fa.disabled",

            Self::OrganizationCreated => "organization.created",
            Self::OrganizationUpdated => "organization.updated",
            Self::OrganizationDeleted => "organization.deleted",

            Self::MemberAdded => "organization.member_added",
            Self::MemberRemoved => "organization.member_removed",
            Self::MemberRoleChanged => "organization.member_role_changed",

            Self::InvitationSent => "organization.invitation_sent",
            Self::InvitationAccepted => "organization.invitation_accepted",
            Self::InvitationRevoked => "organization.invitation_revoked",

            Self::ApiKeyCreated => "api_key.created",
            Self::ApiKeyRevoked => "api_key.revoked",

            Self::MagicLinkSent => "magic_link.sent",
            Self::MagicLinkVerified => "magic_link.verified",

            Self::OtpSent => "otp.sent",
            Self::OtpVerified => "otp.verified",

            Self::PhoneVerified => "phone.verified",
        }
    }
}

impl std::fmt::Display for WebhookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: Uuid,
    pub event: WebhookEvent,
    pub timestamp: DateTime<Utc>,
    pub data: WebhookData,
}

impl WebhookPayload {
    pub fn new(event: WebhookEvent, data: WebhookData) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            timestamp: Utc::now(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebhookData {
    User(UserWebhookData),
    Session(SessionWebhookData),
    Organization(OrganizationWebhookData),
    Member(MemberWebhookData),
    Invitation(InvitationWebhookData),
    ApiKey(ApiKeyWebhookData),
    Auth(AuthWebhookData),
    Empty {},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWebhookData {
    pub user_id: Uuid,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWebhookData {
    pub session_id: Uuid,
    pub user_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationWebhookData {
    pub organization_id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberWebhookData {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationWebhookData {
    pub invitation_id: Uuid,
    pub organization_id: Uuid,
    pub email: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inviter_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyWebhookData {
    pub api_key_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthWebhookData {
    pub user_id: Uuid,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}
