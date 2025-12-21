mod account;
mod account_lockout;
mod api_key;
mod audit_log;
mod impersonation_session;
mod ip_rule;
mod organization;
mod organization_invitation;
mod organization_member;
mod passkey;
mod passkey_challenge;
mod password_history;
mod session;
mod two_factor;
mod user;
mod verification_token;

pub use account::SeaOrmAccountRepository;
pub use account_lockout::SeaOrmAccountLockoutRepository;
pub use api_key::SeaOrmApiKeyRepository;
pub use audit_log::SeaOrmAuditLogRepository;
pub use impersonation_session::SeaOrmImpersonationSessionRepository;
pub use ip_rule::SeaOrmIpRuleRepository;
pub use organization::SeaOrmOrganizationRepository;
pub use organization_invitation::SeaOrmOrganizationInvitationRepository;
pub use organization_member::SeaOrmOrganizationMemberRepository;
pub use passkey::SeaOrmPasskeyRepository;
pub use passkey_challenge::SeaOrmPasskeyChallengeRepository;
pub use password_history::SeaOrmPasswordHistoryRepository;
pub use session::SeaOrmSessionRepository;
pub use two_factor::SeaOrmTwoFactorRepository;
pub use user::SeaOrmUserRepository;
pub use verification_token::SeaOrmVerificationTokenRepository;

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tsa_core::Adapter;

#[derive(Clone)]
pub struct SeaOrmAdapter {
    users: SeaOrmUserRepository,
    sessions: SeaOrmSessionRepository,
    accounts: SeaOrmAccountRepository,
    verification_tokens: SeaOrmVerificationTokenRepository,
    two_factor: SeaOrmTwoFactorRepository,
    organizations: SeaOrmOrganizationRepository,
    organization_members: SeaOrmOrganizationMemberRepository,
    organization_invitations: SeaOrmOrganizationInvitationRepository,
    api_keys: SeaOrmApiKeyRepository,
    passkeys: SeaOrmPasskeyRepository,
    passkey_challenges: SeaOrmPasskeyChallengeRepository,
    audit_logs: SeaOrmAuditLogRepository,
    account_lockouts: SeaOrmAccountLockoutRepository,
    password_history: SeaOrmPasswordHistoryRepository,
    ip_rules: SeaOrmIpRuleRepository,
    impersonation_sessions: SeaOrmImpersonationSessionRepository,
}

impl SeaOrmAdapter {
    pub fn new(db: DatabaseConnection) -> Self {
        let db = Arc::new(db);
        Self {
            users: SeaOrmUserRepository::new(db.clone()),
            sessions: SeaOrmSessionRepository::new(db.clone()),
            accounts: SeaOrmAccountRepository::new(db.clone()),
            verification_tokens: SeaOrmVerificationTokenRepository::new(db.clone()),
            two_factor: SeaOrmTwoFactorRepository::new(db.clone()),
            organizations: SeaOrmOrganizationRepository::new(db.clone()),
            organization_members: SeaOrmOrganizationMemberRepository::new(db.clone()),
            organization_invitations: SeaOrmOrganizationInvitationRepository::new(db.clone()),
            api_keys: SeaOrmApiKeyRepository::new(db.clone()),
            passkeys: SeaOrmPasskeyRepository::new(db.clone()),
            passkey_challenges: SeaOrmPasskeyChallengeRepository::new(db.clone()),
            audit_logs: SeaOrmAuditLogRepository::new(db.clone()),
            account_lockouts: SeaOrmAccountLockoutRepository::new(db.clone()),
            password_history: SeaOrmPasswordHistoryRepository::new(db.clone()),
            ip_rules: SeaOrmIpRuleRepository::new(db.clone()),
            impersonation_sessions: SeaOrmImpersonationSessionRepository::new(db),
        }
    }
}

impl Adapter for SeaOrmAdapter {
    type UserRepo = SeaOrmUserRepository;
    type SessionRepo = SeaOrmSessionRepository;
    type AccountRepo = SeaOrmAccountRepository;
    type VerificationTokenRepo = SeaOrmVerificationTokenRepository;
    type TwoFactorRepo = SeaOrmTwoFactorRepository;
    type OrganizationRepo = SeaOrmOrganizationRepository;
    type OrganizationMemberRepo = SeaOrmOrganizationMemberRepository;
    type OrganizationInvitationRepo = SeaOrmOrganizationInvitationRepository;
    type ApiKeyRepo = SeaOrmApiKeyRepository;
    type PasskeyRepo = SeaOrmPasskeyRepository;
    type PasskeyChallengeRepo = SeaOrmPasskeyChallengeRepository;
    type AuditLogRepo = SeaOrmAuditLogRepository;
    type AccountLockoutRepo = SeaOrmAccountLockoutRepository;
    type PasswordHistoryRepo = SeaOrmPasswordHistoryRepository;
    type IpRuleRepo = SeaOrmIpRuleRepository;
    type ImpersonationSessionRepo = SeaOrmImpersonationSessionRepository;

    fn users(&self) -> &Self::UserRepo {
        &self.users
    }

    fn sessions(&self) -> &Self::SessionRepo {
        &self.sessions
    }

    fn accounts(&self) -> &Self::AccountRepo {
        &self.accounts
    }

    fn verification_tokens(&self) -> &Self::VerificationTokenRepo {
        &self.verification_tokens
    }

    fn two_factor(&self) -> &Self::TwoFactorRepo {
        &self.two_factor
    }

    fn organizations(&self) -> &Self::OrganizationRepo {
        &self.organizations
    }

    fn organization_members(&self) -> &Self::OrganizationMemberRepo {
        &self.organization_members
    }

    fn organization_invitations(&self) -> &Self::OrganizationInvitationRepo {
        &self.organization_invitations
    }

    fn api_keys(&self) -> &Self::ApiKeyRepo {
        &self.api_keys
    }

    fn passkeys(&self) -> &Self::PasskeyRepo {
        &self.passkeys
    }

    fn passkey_challenges(&self) -> &Self::PasskeyChallengeRepo {
        &self.passkey_challenges
    }

    fn audit_logs(&self) -> &Self::AuditLogRepo {
        &self.audit_logs
    }

    fn account_lockouts(&self) -> &Self::AccountLockoutRepo {
        &self.account_lockouts
    }

    fn password_history(&self) -> &Self::PasswordHistoryRepo {
        &self.password_history
    }

    fn ip_rules(&self) -> &Self::IpRuleRepo {
        &self.ip_rules
    }

    fn impersonation_sessions(&self) -> &Self::ImpersonationSessionRepo {
        &self.impersonation_sessions
    }
}
