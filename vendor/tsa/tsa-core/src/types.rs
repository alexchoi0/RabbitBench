use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub name: Option<String>,
    pub image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_account_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    EmailVerification,
    PasswordReset,
    MagicLink,
    EmailOtp,
    PhoneOtp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub enabled: bool,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrganizationRole {
    Owner,
    Admin,
    Member,
}

impl std::fmt::Display for OrganizationRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrganizationRole::Owner => write!(f, "owner"),
            OrganizationRole::Admin => write!(f, "admin"),
            OrganizationRole::Member => write!(f, "member"),
        }
    }
}

impl std::str::FromStr for OrganizationRole {
    type Err = crate::TsaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(OrganizationRole::Owner),
            "admin" => Ok(OrganizationRole::Admin),
            "member" => Ok(OrganizationRole::Member),
            _ => Err(crate::TsaError::InvalidInput(format!(
                "Invalid role: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: OrganizationRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

impl std::fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationStatus::Pending => write!(f, "pending"),
            InvitationStatus::Accepted => write!(f, "accepted"),
            InvitationStatus::Expired => write!(f, "expired"),
            InvitationStatus::Revoked => write!(f, "revoked"),
        }
    }
}

impl std::str::FromStr for InvitationStatus {
    type Err = crate::TsaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(InvitationStatus::Pending),
            "accepted" => Ok(InvitationStatus::Accepted),
            "expired" => Ok(InvitationStatus::Expired),
            "revoked" => Ok(InvitationStatus::Revoked),
            _ => Err(crate::TsaError::InvalidInput(format!(
                "Invalid status: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInvitation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub email: String,
    pub role: OrganizationRole,
    pub token_hash: String,
    pub invited_by: Uuid,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub key_hash: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passkey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: u32,
    pub name: String,
    pub transports: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyChallenge {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub challenge: Vec<u8>,
    pub challenge_type: PasskeyChallengeType,
    pub state: Vec<u8>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PasskeyChallengeType {
    Registration,
    Authentication,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
    Cancelled,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalStatus::Pending => write!(f, "pending"),
            ApprovalStatus::Approved => write!(f, "approved"),
            ApprovalStatus::Denied => write!(f, "denied"),
            ApprovalStatus::Expired => write!(f, "expired"),
            ApprovalStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for ApprovalStatus {
    type Err = crate::TsaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ApprovalStatus::Pending),
            "approved" => Ok(ApprovalStatus::Approved),
            "denied" => Ok(ApprovalStatus::Denied),
            "expired" => Ok(ApprovalStatus::Expired),
            "cancelled" => Ok(ApprovalStatus::Cancelled),
            _ => Err(crate::TsaError::InvalidInput(format!(
                "Invalid approval status: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requester_id: Uuid,
    pub policy_name: String,
    pub permission: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub reason: Option<String>,
    pub context: Option<serde_json::Value>,
    pub status: ApprovalStatus,
    pub required_approvals: u32,
    pub expires_at: DateTime<Utc>,
    pub auto_deny_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalDecision {
    Approved,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub id: Uuid,
    pub request_id: Uuid,
    pub approver_id: Uuid,
    pub decision: ApprovalDecision,
    pub comment: Option<String>,
    pub channel: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalToken {
    pub id: Uuid,
    pub request_id: Uuid,
    pub token_hash: String,
    pub approver_email: Option<String>,
    pub channel: String,
    pub decision: Option<ApprovalDecision>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Signup,
    SigninSuccess,
    SigninFailed,
    Signout,
    PasswordChanged,
    PasswordResetRequested,
    PasswordReset,
    EmailVerified,
    PhoneVerified,
    TwoFactorEnabled,
    TwoFactorDisabled,
    TwoFactorVerified,
    TwoFactorFailed,
    SessionCreated,
    SessionRefreshed,
    SessionRevoked,
    AccountLocked,
    AccountUnlocked,
    ApiKeyCreated,
    ApiKeyRevoked,
    ImpersonationStarted,
    ImpersonationEnded,
    OrganizationCreated,
    OrganizationUpdated,
    OrganizationDeleted,
    MemberAdded,
    MemberRemoved,
    MemberRoleChanged,
    InvitationSent,
    InvitationAccepted,
    InvitationRevoked,
    MagicLinkSent,
    MagicLinkVerified,
    OtpSent,
    OtpVerified,
    PasskeyRegistered,
    PasskeyAuthenticated,
    IpBlocked,
    IpAllowed,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AuditAction::Signup => "signup",
            AuditAction::SigninSuccess => "signin_success",
            AuditAction::SigninFailed => "signin_failed",
            AuditAction::Signout => "signout",
            AuditAction::PasswordChanged => "password_changed",
            AuditAction::PasswordResetRequested => "password_reset_requested",
            AuditAction::PasswordReset => "password_reset",
            AuditAction::EmailVerified => "email_verified",
            AuditAction::PhoneVerified => "phone_verified",
            AuditAction::TwoFactorEnabled => "two_factor_enabled",
            AuditAction::TwoFactorDisabled => "two_factor_disabled",
            AuditAction::TwoFactorVerified => "two_factor_verified",
            AuditAction::TwoFactorFailed => "two_factor_failed",
            AuditAction::SessionCreated => "session_created",
            AuditAction::SessionRefreshed => "session_refreshed",
            AuditAction::SessionRevoked => "session_revoked",
            AuditAction::AccountLocked => "account_locked",
            AuditAction::AccountUnlocked => "account_unlocked",
            AuditAction::ApiKeyCreated => "api_key_created",
            AuditAction::ApiKeyRevoked => "api_key_revoked",
            AuditAction::ImpersonationStarted => "impersonation_started",
            AuditAction::ImpersonationEnded => "impersonation_ended",
            AuditAction::OrganizationCreated => "organization_created",
            AuditAction::OrganizationUpdated => "organization_updated",
            AuditAction::OrganizationDeleted => "organization_deleted",
            AuditAction::MemberAdded => "member_added",
            AuditAction::MemberRemoved => "member_removed",
            AuditAction::MemberRoleChanged => "member_role_changed",
            AuditAction::InvitationSent => "invitation_sent",
            AuditAction::InvitationAccepted => "invitation_accepted",
            AuditAction::InvitationRevoked => "invitation_revoked",
            AuditAction::MagicLinkSent => "magic_link_sent",
            AuditAction::MagicLinkVerified => "magic_link_verified",
            AuditAction::OtpSent => "otp_sent",
            AuditAction::OtpVerified => "otp_verified",
            AuditAction::PasskeyRegistered => "passkey_registered",
            AuditAction::PasskeyAuthenticated => "passkey_authenticated",
            AuditAction::IpBlocked => "ip_blocked",
            AuditAction::IpAllowed => "ip_allowed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub action: AuditAction,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLockout {
    pub id: Uuid,
    pub user_id: Uuid,
    pub failed_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_failed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub max_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special: bool,
    pub special_characters: String,
    pub max_age_days: Option<u32>,
    pub password_history_count: u32,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: false,
            special_characters: "!@#$%^&*()_+-=[]{}|;':\",./<>?".to_string(),
            max_age_days: None,
            password_history_count: 0,
        }
    }
}

impl PasswordPolicy {
    pub fn strict() -> Self {
        Self {
            min_length: 12,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: true,
            special_characters: "!@#$%^&*()_+-=[]{}|;':\",./<>?".to_string(),
            max_age_days: Some(90),
            password_history_count: 5,
        }
    }

    pub fn validate(&self, password: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if password.len() < self.min_length as usize {
            errors.push(format!(
                "Password must be at least {} characters",
                self.min_length
            ));
        }

        if password.len() > self.max_length as usize {
            errors.push(format!(
                "Password must be at most {} characters",
                self.max_length
            ));
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        if self.require_numbers && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.push("Password must contain at least one number".to_string());
        }

        if self.require_special
            && !password
                .chars()
                .any(|c| self.special_characters.contains(c))
        {
            errors.push("Password must contain at least one special character".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IpRuleType {
    Allow,
    Block,
}

impl std::fmt::Display for IpRuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpRuleType::Allow => write!(f, "allow"),
            IpRuleType::Block => write!(f, "block"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRule {
    pub id: Uuid,
    pub ip_pattern: String,
    pub rule_type: IpRuleType,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl IpRule {
    pub fn matches(&self, ip: &str) -> bool {
        if self.ip_pattern == "*" {
            return true;
        }

        if self.ip_pattern.contains('/') {
            return self.matches_cidr(ip);
        }

        if self.ip_pattern.contains('*') {
            return self.matches_wildcard(ip);
        }

        self.ip_pattern == ip
    }

    fn matches_cidr(&self, ip: &str) -> bool {
        use std::net::IpAddr;

        let parts: Vec<&str> = self.ip_pattern.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        let network: IpAddr = match parts[0].parse() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        let prefix_len: u8 = match parts[1].parse() {
            Ok(len) => len,
            Err(_) => return false,
        };

        let target: IpAddr = match ip.parse() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        match (network, target) {
            (IpAddr::V4(net), IpAddr::V4(tgt)) => {
                if prefix_len > 32 {
                    return false;
                }
                let mask = if prefix_len == 0 {
                    0
                } else {
                    !0u32 << (32 - prefix_len)
                };
                (u32::from(net) & mask) == (u32::from(tgt) & mask)
            }
            (IpAddr::V6(net), IpAddr::V6(tgt)) => {
                if prefix_len > 128 {
                    return false;
                }
                let net_bytes = net.octets();
                let tgt_bytes = tgt.octets();
                let full_bytes = (prefix_len / 8) as usize;
                let remaining_bits = prefix_len % 8;

                if net_bytes[..full_bytes] != tgt_bytes[..full_bytes] {
                    return false;
                }

                if remaining_bits > 0 && full_bytes < 16 {
                    let mask = !0u8 << (8 - remaining_bits);
                    if (net_bytes[full_bytes] & mask) != (tgt_bytes[full_bytes] & mask) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn matches_wildcard(&self, ip: &str) -> bool {
        let pattern_parts: Vec<&str> = self.ip_pattern.split('.').collect();
        let ip_parts: Vec<&str> = ip.split('.').collect();

        if pattern_parts.len() != ip_parts.len() {
            return false;
        }

        for (pattern_part, ip_part) in pattern_parts.iter().zip(ip_parts.iter()) {
            if *pattern_part != "*" && *pattern_part != *ip_part {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpersonationSession {
    pub id: Uuid,
    pub admin_id: Uuid,
    pub target_user_id: Uuid,
    pub original_session_id: Uuid,
    pub impersonation_session_id: Uuid,
    pub reason: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_role_display() {
        assert_eq!(OrganizationRole::Owner.to_string(), "owner");
        assert_eq!(OrganizationRole::Admin.to_string(), "admin");
        assert_eq!(OrganizationRole::Member.to_string(), "member");
    }

    #[test]
    fn test_organization_role_from_str() {
        assert_eq!(
            "owner".parse::<OrganizationRole>().unwrap(),
            OrganizationRole::Owner
        );
        assert_eq!(
            "ADMIN".parse::<OrganizationRole>().unwrap(),
            OrganizationRole::Admin
        );
        assert_eq!(
            "Member".parse::<OrganizationRole>().unwrap(),
            OrganizationRole::Member
        );
        assert!("invalid".parse::<OrganizationRole>().is_err());
    }

    #[test]
    fn test_invitation_status_display() {
        assert_eq!(InvitationStatus::Pending.to_string(), "pending");
        assert_eq!(InvitationStatus::Accepted.to_string(), "accepted");
        assert_eq!(InvitationStatus::Expired.to_string(), "expired");
        assert_eq!(InvitationStatus::Revoked.to_string(), "revoked");
    }

    #[test]
    fn test_invitation_status_from_str() {
        assert_eq!(
            "pending".parse::<InvitationStatus>().unwrap(),
            InvitationStatus::Pending
        );
        assert_eq!(
            "ACCEPTED".parse::<InvitationStatus>().unwrap(),
            InvitationStatus::Accepted
        );
        assert!("invalid".parse::<InvitationStatus>().is_err());
    }

    #[test]
    fn test_approval_status_display() {
        assert_eq!(ApprovalStatus::Pending.to_string(), "pending");
        assert_eq!(ApprovalStatus::Approved.to_string(), "approved");
        assert_eq!(ApprovalStatus::Denied.to_string(), "denied");
    }

    #[test]
    fn test_approval_status_from_str() {
        assert_eq!(
            "pending".parse::<ApprovalStatus>().unwrap(),
            ApprovalStatus::Pending
        );
        assert_eq!(
            "APPROVED".parse::<ApprovalStatus>().unwrap(),
            ApprovalStatus::Approved
        );
        assert!("unknown".parse::<ApprovalStatus>().is_err());
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            email_verified: true,
            phone: Some("+1234567890".to_string()),
            phone_verified: false,
            name: Some("Test User".to_string()),
            image: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.email, deserialized.email);
        assert_eq!(user.phone, deserialized.phone);
    }

    #[test]
    fn test_session_serialization() {
        let session = Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() + chrono::Duration::days(1),
            created_at: Utc::now(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.user_id, deserialized.user_id);
    }

    #[test]
    fn test_token_type_equality() {
        assert_eq!(TokenType::EmailVerification, TokenType::EmailVerification);
        assert_ne!(TokenType::EmailVerification, TokenType::PasswordReset);
        assert_ne!(TokenType::MagicLink, TokenType::EmailOtp);
    }

    #[test]
    fn test_api_key_serialization() {
        let api_key = ApiKey {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            organization_id: Some(Uuid::new_v4()),
            name: "Test Key".to_string(),
            key_hash: "hash".to_string(),
            prefix: "tsa_abc".to_string(),
            scopes: vec!["read:users".to_string(), "write:users".to_string()],
            expires_at: None,
            last_used_at: None,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&api_key).unwrap();
        let deserialized: ApiKey = serde_json::from_str(&json).unwrap();

        assert_eq!(api_key.scopes, deserialized.scopes);
        assert_eq!(api_key.prefix, deserialized.prefix);
    }
}
