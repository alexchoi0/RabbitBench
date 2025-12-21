use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;
use tsa_core::{
    Account, AccountLockout, AccountLockoutRepository, AccountRepository, Adapter, ApiKey,
    ApiKeyRepository, AuditAction, AuditLog, AuditLogRepository, ImpersonationSession,
    ImpersonationSessionRepository, InvitationStatus, IpRule, IpRuleRepository, IpRuleType,
    Organization, OrganizationInvitation, OrganizationInvitationRepository, OrganizationMember,
    OrganizationMemberRepository, OrganizationRepository, Passkey, PasskeyChallenge,
    PasskeyChallengeRepository, PasskeyRepository, PasswordHistory, PasswordHistoryRepository,
    Result, Session, SessionRepository, TsaError, TwoFactor, TwoFactorRepository, User,
    UserRepository, VerificationToken, VerificationTokenRepository,
};
use uuid::Uuid;

pub struct InMemoryAdapter {
    users: InMemoryUserRepository,
    sessions: InMemorySessionRepository,
    accounts: InMemoryAccountRepository,
    verification_tokens: InMemoryVerificationTokenRepository,
    two_factor: InMemoryTwoFactorRepository,
    organizations: InMemoryOrganizationRepository,
    organization_members: InMemoryOrganizationMemberRepository,
    organization_invitations: InMemoryOrganizationInvitationRepository,
    api_keys: InMemoryApiKeyRepository,
    passkeys: InMemoryPasskeyRepository,
    passkey_challenges: InMemoryPasskeyChallengeRepository,
    audit_logs: InMemoryAuditLogRepository,
    account_lockouts: InMemoryAccountLockoutRepository,
    password_history: InMemoryPasswordHistoryRepository,
    ip_rules: InMemoryIpRuleRepository,
    impersonation_sessions: InMemoryImpersonationSessionRepository,
}

impl InMemoryAdapter {
    pub fn new() -> Self {
        Self {
            users: InMemoryUserRepository::new(),
            sessions: InMemorySessionRepository::new(),
            accounts: InMemoryAccountRepository::new(),
            verification_tokens: InMemoryVerificationTokenRepository::new(),
            two_factor: InMemoryTwoFactorRepository::new(),
            organizations: InMemoryOrganizationRepository::new(),
            organization_members: InMemoryOrganizationMemberRepository::new(),
            organization_invitations: InMemoryOrganizationInvitationRepository::new(),
            api_keys: InMemoryApiKeyRepository::new(),
            passkeys: InMemoryPasskeyRepository::new(),
            passkey_challenges: InMemoryPasskeyChallengeRepository::new(),
            audit_logs: InMemoryAuditLogRepository::new(),
            account_lockouts: InMemoryAccountLockoutRepository::new(),
            password_history: InMemoryPasswordHistoryRepository::new(),
            ip_rules: InMemoryIpRuleRepository::new(),
            impersonation_sessions: InMemoryImpersonationSessionRepository::new(),
        }
    }
}

impl Default for InMemoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for InMemoryAdapter {
    type UserRepo = InMemoryUserRepository;
    type SessionRepo = InMemorySessionRepository;
    type AccountRepo = InMemoryAccountRepository;
    type VerificationTokenRepo = InMemoryVerificationTokenRepository;
    type TwoFactorRepo = InMemoryTwoFactorRepository;
    type OrganizationRepo = InMemoryOrganizationRepository;
    type OrganizationMemberRepo = InMemoryOrganizationMemberRepository;
    type OrganizationInvitationRepo = InMemoryOrganizationInvitationRepository;
    type ApiKeyRepo = InMemoryApiKeyRepository;
    type PasskeyRepo = InMemoryPasskeyRepository;
    type PasskeyChallengeRepo = InMemoryPasskeyChallengeRepository;
    type AuditLogRepo = InMemoryAuditLogRepository;
    type AccountLockoutRepo = InMemoryAccountLockoutRepository;
    type PasswordHistoryRepo = InMemoryPasswordHistoryRepository;
    type IpRuleRepo = InMemoryIpRuleRepository;
    type ImpersonationSessionRepo = InMemoryImpersonationSessionRepository;

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

pub struct InMemoryUserRepository {
    users: RwLock<HashMap<Uuid, User>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, user: &User) -> Result<User> {
        let mut users = self.users.write().unwrap();
        if users.values().any(|u| u.email == user.email) {
            return Err(TsaError::UserAlreadyExists);
        }
        users.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let users = self.users.read().unwrap();
        Ok(users.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let users = self.users.read().unwrap();
        Ok(users.values().find(|u| u.email == email).cloned())
    }

    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>> {
        let users = self.users.read().unwrap();
        Ok(users
            .values()
            .find(|u| u.phone.as_deref() == Some(phone))
            .cloned())
    }

    async fn update(&self, user: &User) -> Result<User> {
        let mut users = self.users.write().unwrap();
        users.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut users = self.users.write().unwrap();
        users.remove(&id);
        Ok(())
    }
}

pub struct InMemorySessionRepository {
    sessions: RwLock<HashMap<Uuid, Session>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn create(&self, session: &Session) -> Result<Session> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id, session.clone());
        Ok(session.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions.get(&id).cloned())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .find(|s| s.token_hash == token_hash)
            .cloned())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn update(&self, session: &Session) -> Result<Session> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id, session.clone());
        Ok(session.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(&id);
        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, s| s.user_id != user_id);
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let mut sessions = self.sessions.write().unwrap();
        let now = Utc::now();
        let before = sessions.len();
        sessions.retain(|_, s| s.expires_at > now);
        Ok((before - sessions.len()) as u64)
    }
}

pub struct InMemoryAccountRepository {
    accounts: RwLock<HashMap<Uuid, Account>>,
}

impl InMemoryAccountRepository {
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountRepository for InMemoryAccountRepository {
    async fn create(&self, account: &Account) -> Result<Account> {
        let mut accounts = self.accounts.write().unwrap();
        accounts.insert(account.id, account.clone());
        Ok(account.clone())
    }

    async fn find_by_provider(
        &self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>> {
        let accounts = self.accounts.read().unwrap();
        Ok(accounts
            .values()
            .find(|a| a.provider == provider && a.provider_account_id == provider_account_id)
            .cloned())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Account>> {
        let accounts = self.accounts.read().unwrap();
        Ok(accounts
            .values()
            .filter(|a| a.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut accounts = self.accounts.write().unwrap();
        accounts.remove(&id);
        Ok(())
    }
}

pub struct InMemoryVerificationTokenRepository {
    tokens: RwLock<HashMap<Uuid, VerificationToken>>,
}

impl InMemoryVerificationTokenRepository {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryVerificationTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VerificationTokenRepository for InMemoryVerificationTokenRepository {
    async fn create(&self, token: &VerificationToken) -> Result<VerificationToken> {
        let mut tokens = self.tokens.write().unwrap();
        tokens.insert(token.id, token.clone());
        Ok(token.clone())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<VerificationToken>> {
        let tokens = self.tokens.read().unwrap();
        Ok(tokens
            .values()
            .find(|t| t.token_hash == token_hash)
            .cloned())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut tokens = self.tokens.write().unwrap();
        tokens.remove(&id);
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let mut tokens = self.tokens.write().unwrap();
        let now = Utc::now();
        let before = tokens.len();
        tokens.retain(|_, t| t.expires_at > now);
        Ok((before - tokens.len()) as u64)
    }
}

pub struct InMemoryTwoFactorRepository {
    two_factors: RwLock<HashMap<Uuid, TwoFactor>>,
}

impl InMemoryTwoFactorRepository {
    pub fn new() -> Self {
        Self {
            two_factors: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryTwoFactorRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TwoFactorRepository for InMemoryTwoFactorRepository {
    async fn create(&self, two_factor: &TwoFactor) -> Result<TwoFactor> {
        let mut two_factors = self.two_factors.write().unwrap();
        two_factors.insert(two_factor.id, two_factor.clone());
        Ok(two_factor.clone())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<TwoFactor>> {
        let two_factors = self.two_factors.read().unwrap();
        Ok(two_factors.values().find(|t| t.user_id == user_id).cloned())
    }

    async fn update(&self, two_factor: &TwoFactor) -> Result<TwoFactor> {
        let mut two_factors = self.two_factors.write().unwrap();
        two_factors.insert(two_factor.id, two_factor.clone());
        Ok(two_factor.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut two_factors = self.two_factors.write().unwrap();
        two_factors.remove(&id);
        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<()> {
        let mut two_factors = self.two_factors.write().unwrap();
        two_factors.retain(|_, t| t.user_id != user_id);
        Ok(())
    }
}

pub struct InMemoryOrganizationRepository {
    organizations: RwLock<HashMap<Uuid, Organization>>,
}

impl InMemoryOrganizationRepository {
    pub fn new() -> Self {
        Self {
            organizations: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryOrganizationRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OrganizationRepository for InMemoryOrganizationRepository {
    async fn create(&self, organization: &Organization) -> Result<Organization> {
        let mut orgs = self.organizations.write().unwrap();
        if orgs.values().any(|o| o.slug == organization.slug) {
            return Err(TsaError::OrganizationAlreadyExists);
        }
        orgs.insert(organization.id, organization.clone());
        Ok(organization.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>> {
        let orgs = self.organizations.read().unwrap();
        Ok(orgs.get(&id).cloned())
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>> {
        let orgs = self.organizations.read().unwrap();
        Ok(orgs.values().find(|o| o.slug == slug).cloned())
    }

    async fn update(&self, organization: &Organization) -> Result<Organization> {
        let mut orgs = self.organizations.write().unwrap();
        orgs.insert(organization.id, organization.clone());
        Ok(organization.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut orgs = self.organizations.write().unwrap();
        orgs.remove(&id);
        Ok(())
    }
}

pub struct InMemoryOrganizationMemberRepository {
    members: RwLock<HashMap<Uuid, OrganizationMember>>,
}

impl InMemoryOrganizationMemberRepository {
    pub fn new() -> Self {
        Self {
            members: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryOrganizationMemberRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OrganizationMemberRepository for InMemoryOrganizationMemberRepository {
    async fn create(&self, member: &OrganizationMember) -> Result<OrganizationMember> {
        let mut members = self.members.write().unwrap();
        if members
            .values()
            .any(|m| m.organization_id == member.organization_id && m.user_id == member.user_id)
        {
            return Err(TsaError::AlreadyOrganizationMember);
        }
        members.insert(member.id, member.clone());
        Ok(member.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationMember>> {
        let members = self.members.read().unwrap();
        Ok(members.get(&id).cloned())
    }

    async fn find_by_org_and_user(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<OrganizationMember>> {
        let members = self.members.read().unwrap();
        Ok(members
            .values()
            .find(|m| m.organization_id == organization_id && m.user_id == user_id)
            .cloned())
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<OrganizationMember>> {
        let members = self.members.read().unwrap();
        Ok(members
            .values()
            .filter(|m| m.organization_id == organization_id)
            .cloned()
            .collect())
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<OrganizationMember>> {
        let members = self.members.read().unwrap();
        Ok(members
            .values()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn update(&self, member: &OrganizationMember) -> Result<OrganizationMember> {
        let mut members = self.members.write().unwrap();
        members.insert(member.id, member.clone());
        Ok(member.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut members = self.members.write().unwrap();
        members.remove(&id);
        Ok(())
    }

    async fn delete_by_organization(&self, organization_id: Uuid) -> Result<()> {
        let mut members = self.members.write().unwrap();
        members.retain(|_, m| m.organization_id != organization_id);
        Ok(())
    }
}

pub struct InMemoryOrganizationInvitationRepository {
    invitations: RwLock<HashMap<Uuid, OrganizationInvitation>>,
}

impl InMemoryOrganizationInvitationRepository {
    pub fn new() -> Self {
        Self {
            invitations: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryOrganizationInvitationRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OrganizationInvitationRepository for InMemoryOrganizationInvitationRepository {
    async fn create(&self, invitation: &OrganizationInvitation) -> Result<OrganizationInvitation> {
        let mut invitations = self.invitations.write().unwrap();
        invitations.insert(invitation.id, invitation.clone());
        Ok(invitation.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationInvitation>> {
        let invitations = self.invitations.read().unwrap();
        Ok(invitations.get(&id).cloned())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<OrganizationInvitation>> {
        let invitations = self.invitations.read().unwrap();
        Ok(invitations
            .values()
            .find(|i| i.token_hash == token_hash)
            .cloned())
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OrganizationInvitation>> {
        let invitations = self.invitations.read().unwrap();
        Ok(invitations
            .values()
            .filter(|i| i.organization_id == organization_id)
            .cloned()
            .collect())
    }

    async fn find_by_email(&self, email: &str) -> Result<Vec<OrganizationInvitation>> {
        let invitations = self.invitations.read().unwrap();
        Ok(invitations
            .values()
            .filter(|i| i.email == email)
            .cloned()
            .collect())
    }

    async fn find_pending_by_org_and_email(
        &self,
        organization_id: Uuid,
        email: &str,
    ) -> Result<Option<OrganizationInvitation>> {
        let invitations = self.invitations.read().unwrap();
        Ok(invitations
            .values()
            .find(|i| {
                i.organization_id == organization_id
                    && i.email == email
                    && i.status == InvitationStatus::Pending
            })
            .cloned())
    }

    async fn update(&self, invitation: &OrganizationInvitation) -> Result<OrganizationInvitation> {
        let mut invitations = self.invitations.write().unwrap();
        invitations.insert(invitation.id, invitation.clone());
        Ok(invitation.clone())
    }

    async fn update_status(&self, id: Uuid, status: InvitationStatus) -> Result<()> {
        let mut invitations = self.invitations.write().unwrap();
        if let Some(invitation) = invitations.get_mut(&id) {
            invitation.status = status;
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut invitations = self.invitations.write().unwrap();
        invitations.remove(&id);
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let mut invitations = self.invitations.write().unwrap();
        let now = Utc::now();
        let before = invitations.len();
        invitations.retain(|_, i| i.expires_at > now || i.status != InvitationStatus::Pending);
        Ok((before - invitations.len()) as u64)
    }
}

pub struct InMemoryApiKeyRepository {
    api_keys: RwLock<HashMap<Uuid, ApiKey>>,
}

impl InMemoryApiKeyRepository {
    pub fn new() -> Self {
        Self {
            api_keys: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryApiKeyRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ApiKeyRepository for InMemoryApiKeyRepository {
    async fn create(&self, api_key: &ApiKey) -> Result<ApiKey> {
        let mut api_keys = self.api_keys.write().unwrap();
        api_keys.insert(api_key.id, api_key.clone());
        Ok(api_key.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKey>> {
        let api_keys = self.api_keys.read().unwrap();
        Ok(api_keys.get(&id).cloned())
    }

    async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let api_keys = self.api_keys.read().unwrap();
        Ok(api_keys.values().find(|k| k.key_hash == key_hash).cloned())
    }

    async fn find_by_prefix(&self, prefix: &str) -> Result<Option<ApiKey>> {
        let api_keys = self.api_keys.read().unwrap();
        Ok(api_keys.values().find(|k| k.prefix == prefix).cloned())
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let api_keys = self.api_keys.read().unwrap();
        Ok(api_keys
            .values()
            .filter(|k| k.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<ApiKey>> {
        let api_keys = self.api_keys.read().unwrap();
        Ok(api_keys
            .values()
            .filter(|k| k.organization_id == Some(organization_id))
            .cloned()
            .collect())
    }

    async fn update(&self, api_key: &ApiKey) -> Result<ApiKey> {
        let mut api_keys = self.api_keys.write().unwrap();
        api_keys.insert(api_key.id, api_key.clone());
        Ok(api_key.clone())
    }

    async fn update_last_used(&self, id: Uuid) -> Result<()> {
        let mut api_keys = self.api_keys.write().unwrap();
        if let Some(api_key) = api_keys.get_mut(&id) {
            api_key.last_used_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut api_keys = self.api_keys.write().unwrap();
        api_keys.remove(&id);
        Ok(())
    }

    async fn delete_by_user(&self, user_id: Uuid) -> Result<()> {
        let mut api_keys = self.api_keys.write().unwrap();
        api_keys.retain(|_, k| k.user_id != user_id);
        Ok(())
    }
}

pub struct InMemoryPasskeyRepository {
    passkeys: RwLock<HashMap<Uuid, Passkey>>,
}

impl InMemoryPasskeyRepository {
    pub fn new() -> Self {
        Self {
            passkeys: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPasskeyRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PasskeyRepository for InMemoryPasskeyRepository {
    async fn create(&self, passkey: &Passkey) -> Result<Passkey> {
        let mut passkeys = self.passkeys.write().unwrap();
        if passkeys
            .values()
            .any(|p| p.credential_id == passkey.credential_id)
        {
            return Err(TsaError::PasskeyAlreadyRegistered);
        }
        passkeys.insert(passkey.id, passkey.clone());
        Ok(passkey.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Passkey>> {
        let passkeys = self.passkeys.read().unwrap();
        Ok(passkeys.get(&id).cloned())
    }

    async fn find_by_credential_id(&self, credential_id: &[u8]) -> Result<Option<Passkey>> {
        let passkeys = self.passkeys.read().unwrap();
        Ok(passkeys
            .values()
            .find(|p| p.credential_id == credential_id)
            .cloned())
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Passkey>> {
        let passkeys = self.passkeys.read().unwrap();
        Ok(passkeys
            .values()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn update(&self, passkey: &Passkey) -> Result<Passkey> {
        let mut passkeys = self.passkeys.write().unwrap();
        passkeys.insert(passkey.id, passkey.clone());
        Ok(passkey.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut passkeys = self.passkeys.write().unwrap();
        passkeys.remove(&id);
        Ok(())
    }

    async fn delete_by_user(&self, user_id: Uuid) -> Result<()> {
        let mut passkeys = self.passkeys.write().unwrap();
        passkeys.retain(|_, p| p.user_id != user_id);
        Ok(())
    }
}

pub struct InMemoryPasskeyChallengeRepository {
    challenges: RwLock<HashMap<Uuid, PasskeyChallenge>>,
}

impl InMemoryPasskeyChallengeRepository {
    pub fn new() -> Self {
        Self {
            challenges: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPasskeyChallengeRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PasskeyChallengeRepository for InMemoryPasskeyChallengeRepository {
    async fn create(&self, challenge: &PasskeyChallenge) -> Result<PasskeyChallenge> {
        let mut challenges = self.challenges.write().unwrap();
        challenges.insert(challenge.id, challenge.clone());
        Ok(challenge.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<PasskeyChallenge>> {
        let challenges = self.challenges.read().unwrap();
        Ok(challenges.get(&id).cloned())
    }

    async fn find_by_challenge(&self, challenge: &[u8]) -> Result<Option<PasskeyChallenge>> {
        let challenges = self.challenges.read().unwrap();
        Ok(challenges
            .values()
            .find(|c| c.challenge == challenge)
            .cloned())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut challenges = self.challenges.write().unwrap();
        challenges.remove(&id);
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let mut challenges = self.challenges.write().unwrap();
        let now = Utc::now();
        let before = challenges.len();
        challenges.retain(|_, c| c.expires_at > now);
        Ok((before - challenges.len()) as u64)
    }
}

pub struct InMemoryAuditLogRepository {
    logs: RwLock<HashMap<Uuid, AuditLog>>,
}

impl InMemoryAuditLogRepository {
    pub fn new() -> Self {
        Self {
            logs: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAuditLogRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditLogRepository for InMemoryAuditLogRepository {
    async fn create(&self, log: &AuditLog) -> Result<AuditLog> {
        let mut logs = self.logs.write().unwrap();
        logs.insert(log.id, log.clone());
        Ok(log.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLog>> {
        let logs = self.logs.read().unwrap();
        Ok(logs.get(&id).cloned())
    }

    async fn find_by_user(&self, user_id: Uuid, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().unwrap();
        let mut result: Vec<_> = logs
            .values()
            .filter(|l| l.user_id == Some(user_id))
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    async fn find_by_action(
        &self,
        action: AuditAction,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().unwrap();
        let mut result: Vec<_> = logs
            .values()
            .filter(|l| l.action == action)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    async fn find_by_ip(&self, ip_address: &str, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().unwrap();
        let mut result: Vec<_> = logs
            .values()
            .filter(|l| l.ip_address.as_deref() == Some(ip_address))
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    async fn find_recent(&self, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().unwrap();
        let mut result: Vec<_> = logs.values().cloned().collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    async fn find_failed(&self, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().unwrap();
        let mut result: Vec<_> = logs.values().filter(|l| !l.success).cloned().collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    async fn count_by_user(&self, user_id: Uuid) -> Result<u64> {
        let logs = self.logs.read().unwrap();
        Ok(logs.values().filter(|l| l.user_id == Some(user_id)).count() as u64)
    }

    async fn count_failed_by_user_since(
        &self,
        user_id: Uuid,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<u32> {
        let logs = self.logs.read().unwrap();
        Ok(logs
            .values()
            .filter(|l| l.user_id == Some(user_id) && !l.success && l.created_at >= since)
            .count() as u32)
    }

    async fn delete_older_than(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64> {
        let mut logs = self.logs.write().unwrap();
        let before_count = logs.len();
        logs.retain(|_, l| l.created_at >= before);
        Ok((before_count - logs.len()) as u64)
    }
}

pub struct InMemoryAccountLockoutRepository {
    lockouts: RwLock<HashMap<Uuid, AccountLockout>>,
}

impl InMemoryAccountLockoutRepository {
    pub fn new() -> Self {
        Self {
            lockouts: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAccountLockoutRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountLockoutRepository for InMemoryAccountLockoutRepository {
    async fn create(&self, lockout: &AccountLockout) -> Result<AccountLockout> {
        let mut lockouts = self.lockouts.write().unwrap();
        lockouts.insert(lockout.id, lockout.clone());
        Ok(lockout.clone())
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Option<AccountLockout>> {
        let lockouts = self.lockouts.read().unwrap();
        Ok(lockouts.values().find(|l| l.user_id == user_id).cloned())
    }

    async fn update(&self, lockout: &AccountLockout) -> Result<AccountLockout> {
        let mut lockouts = self.lockouts.write().unwrap();
        lockouts.insert(lockout.id, lockout.clone());
        Ok(lockout.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut lockouts = self.lockouts.write().unwrap();
        lockouts.remove(&id);
        Ok(())
    }

    async fn delete_by_user(&self, user_id: Uuid) -> Result<()> {
        let mut lockouts = self.lockouts.write().unwrap();
        lockouts.retain(|_, l| l.user_id != user_id);
        Ok(())
    }

    async fn increment_failed_attempts(&self, user_id: Uuid) -> Result<AccountLockout> {
        let mut lockouts = self.lockouts.write().unwrap();
        let now = Utc::now();

        if let Some(lockout) = lockouts.values_mut().find(|l| l.user_id == user_id) {
            lockout.failed_attempts += 1;
            lockout.last_failed_at = Some(now);
            lockout.updated_at = now;
            Ok(lockout.clone())
        } else {
            let lockout = AccountLockout {
                id: Uuid::new_v4(),
                user_id,
                failed_attempts: 1,
                locked_until: None,
                last_failed_at: Some(now),
                created_at: now,
                updated_at: now,
            };
            lockouts.insert(lockout.id, lockout.clone());
            Ok(lockout)
        }
    }

    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<()> {
        let mut lockouts = self.lockouts.write().unwrap();
        if let Some(lockout) = lockouts.values_mut().find(|l| l.user_id == user_id) {
            lockout.failed_attempts = 0;
            lockout.locked_until = None;
            lockout.updated_at = Utc::now();
        }
        Ok(())
    }
}

pub struct InMemoryPasswordHistoryRepository {
    history: RwLock<HashMap<Uuid, PasswordHistory>>,
}

impl InMemoryPasswordHistoryRepository {
    pub fn new() -> Self {
        Self {
            history: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPasswordHistoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PasswordHistoryRepository for InMemoryPasswordHistoryRepository {
    async fn create(&self, entry: &PasswordHistory) -> Result<PasswordHistory> {
        let mut history = self.history.write().unwrap();
        history.insert(entry.id, entry.clone());
        Ok(entry.clone())
    }

    async fn find_by_user(&self, user_id: Uuid, limit: u32) -> Result<Vec<PasswordHistory>> {
        let history = self.history.read().unwrap();
        let mut result: Vec<_> = history
            .values()
            .filter(|h| h.user_id == user_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result.into_iter().take(limit as usize).collect())
    }

    async fn delete_old_entries(&self, user_id: Uuid, keep_count: u32) -> Result<u64> {
        let mut history = self.history.write().unwrap();
        let mut user_entries: Vec<_> = history
            .values()
            .filter(|h| h.user_id == user_id)
            .cloned()
            .collect();
        user_entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let to_delete: Vec<Uuid> = user_entries
            .into_iter()
            .skip(keep_count as usize)
            .map(|h| h.id)
            .collect();

        let count = to_delete.len() as u64;
        for id in to_delete {
            history.remove(&id);
        }
        Ok(count)
    }

    async fn delete_by_user(&self, user_id: Uuid) -> Result<()> {
        let mut history = self.history.write().unwrap();
        history.retain(|_, h| h.user_id != user_id);
        Ok(())
    }
}

pub struct InMemoryIpRuleRepository {
    rules: RwLock<HashMap<Uuid, IpRule>>,
}

impl InMemoryIpRuleRepository {
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryIpRuleRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IpRuleRepository for InMemoryIpRuleRepository {
    async fn create(&self, rule: &IpRule) -> Result<IpRule> {
        let mut rules = self.rules.write().unwrap();
        rules.insert(rule.id, rule.clone());
        Ok(rule.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<IpRule>> {
        let rules = self.rules.read().unwrap();
        Ok(rules.get(&id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<IpRule>> {
        let rules = self.rules.read().unwrap();
        Ok(rules.values().cloned().collect())
    }

    async fn find_by_type(&self, rule_type: IpRuleType) -> Result<Vec<IpRule>> {
        let rules = self.rules.read().unwrap();
        Ok(rules
            .values()
            .filter(|r| r.rule_type == rule_type)
            .cloned()
            .collect())
    }

    async fn find_active(&self) -> Result<Vec<IpRule>> {
        let rules = self.rules.read().unwrap();
        let now = Utc::now();
        Ok(rules
            .values()
            .filter(|r| r.expires_at.is_none_or(|exp| exp > now))
            .cloned()
            .collect())
    }

    async fn update(&self, rule: &IpRule) -> Result<IpRule> {
        let mut rules = self.rules.write().unwrap();
        rules.insert(rule.id, rule.clone());
        Ok(rule.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut rules = self.rules.write().unwrap();
        rules.remove(&id);
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let mut rules = self.rules.write().unwrap();
        let now = Utc::now();
        let before = rules.len();
        rules.retain(|_, r| r.expires_at.is_none_or(|exp| exp > now));
        Ok((before - rules.len()) as u64)
    }
}

pub struct InMemoryImpersonationSessionRepository {
    sessions: RwLock<HashMap<Uuid, ImpersonationSession>>,
}

impl InMemoryImpersonationSessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryImpersonationSessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImpersonationSessionRepository for InMemoryImpersonationSessionRepository {
    async fn create(&self, session: &ImpersonationSession) -> Result<ImpersonationSession> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id, session.clone());
        Ok(session.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ImpersonationSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions.get(&id).cloned())
    }

    async fn find_by_session_id(&self, session_id: Uuid) -> Result<Option<ImpersonationSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .find(|s| s.impersonation_session_id == session_id)
            .cloned())
    }

    async fn find_active_by_admin(&self, admin_id: Uuid) -> Result<Vec<ImpersonationSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .filter(|s| s.admin_id == admin_id && s.ended_at.is_none())
            .cloned()
            .collect())
    }

    async fn find_by_target_user(&self, target_user_id: Uuid) -> Result<Vec<ImpersonationSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .filter(|s| s.target_user_id == target_user_id)
            .cloned()
            .collect())
    }

    async fn end_session(
        &self,
        id: Uuid,
        ended_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<ImpersonationSession> {
        let mut sessions = self.sessions.write().unwrap();
        if let Some(session) = sessions.get_mut(&id) {
            session.ended_at = Some(ended_at);
            Ok(session.clone())
        } else {
            Err(TsaError::SessionNotFound)
        }
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(&id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tsa_core::{OrganizationRole, PasskeyChallengeType, TokenType};

    fn test_user() -> User {
        let now = Utc::now();
        User {
            id: Uuid::new_v4(),
            email: format!("test-{}@example.com", Uuid::new_v4()),
            email_verified: false,
            phone: None,
            phone_verified: false,
            name: Some("Test User".to_string()),
            image: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn test_session(user_id: Uuid) -> Session {
        let now = Utc::now();
        Session {
            id: Uuid::new_v4(),
            user_id,
            token_hash: format!("hash_{}", Uuid::new_v4()),
            expires_at: now + Duration::hours(24),
            created_at: now,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
        }
    }

    fn test_organization() -> Organization {
        let now = Utc::now();
        Organization {
            id: Uuid::new_v4(),
            name: "Test Org".to_string(),
            slug: format!("test-org-{}", Uuid::new_v4()),
            logo: None,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_adapter_default() {
        let adapter = InMemoryAdapter::default();
        assert!(adapter
            .users()
            .find_by_email("nonexistent@test.com")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_user_create() {
        let repo = InMemoryUserRepository::new();
        let user = test_user();

        let created = repo.create(&user).await.unwrap();
        assert_eq!(created.id, user.id);
        assert_eq!(created.email, user.email);
    }

    #[tokio::test]
    async fn test_user_duplicate_email() {
        let repo = InMemoryUserRepository::new();
        let user1 = test_user();
        let mut user2 = test_user();
        user2.email = user1.email.clone();

        repo.create(&user1).await.unwrap();
        let result = repo.create(&user2).await;

        assert!(matches!(result, Err(TsaError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn test_user_find_by_id() {
        let repo = InMemoryUserRepository::new();
        let user = test_user();
        repo.create(&user).await.unwrap();

        let found = repo.find_by_id(user.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);

        let not_found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_user_find_by_email() {
        let repo = InMemoryUserRepository::new();
        let user = test_user();
        repo.create(&user).await.unwrap();

        let found = repo.find_by_email(&user.email).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, user.email);
    }

    #[tokio::test]
    async fn test_user_find_by_phone() {
        let repo = InMemoryUserRepository::new();
        let mut user = test_user();
        user.phone = Some("+1234567890".to_string());
        repo.create(&user).await.unwrap();

        let found = repo.find_by_phone("+1234567890").await.unwrap();
        assert!(found.is_some());

        let not_found = repo.find_by_phone("+9999999999").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_user_update() {
        let repo = InMemoryUserRepository::new();
        let mut user = test_user();
        repo.create(&user).await.unwrap();

        user.name = Some("Updated Name".to_string());
        user.email_verified = true;
        let updated = repo.update(&user).await.unwrap();

        assert_eq!(updated.name, Some("Updated Name".to_string()));
        assert!(updated.email_verified);
    }

    #[tokio::test]
    async fn test_user_delete() {
        let repo = InMemoryUserRepository::new();
        let user = test_user();
        repo.create(&user).await.unwrap();

        repo.delete(user.id).await.unwrap();
        let found = repo.find_by_id(user.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_session_create_and_find() {
        let repo = InMemorySessionRepository::new();
        let user_id = Uuid::new_v4();
        let session = test_session(user_id);

        repo.create(&session).await.unwrap();

        let found = repo.find_by_id(session.id).await.unwrap();
        assert!(found.is_some());

        let found_by_hash = repo.find_by_token_hash(&session.token_hash).await.unwrap();
        assert!(found_by_hash.is_some());
    }

    #[tokio::test]
    async fn test_session_find_by_user_id() {
        let repo = InMemorySessionRepository::new();
        let user_id = Uuid::new_v4();

        repo.create(&test_session(user_id)).await.unwrap();
        repo.create(&test_session(user_id)).await.unwrap();
        repo.create(&test_session(Uuid::new_v4())).await.unwrap();

        let sessions = repo.find_by_user_id(user_id).await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_session_delete_by_user_id() {
        let repo = InMemorySessionRepository::new();
        let user_id = Uuid::new_v4();

        repo.create(&test_session(user_id)).await.unwrap();
        repo.create(&test_session(user_id)).await.unwrap();

        repo.delete_by_user_id(user_id).await.unwrap();
        let sessions = repo.find_by_user_id(user_id).await.unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_session_delete_expired() {
        let repo = InMemorySessionRepository::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let mut expired_session = test_session(user_id);
        expired_session.expires_at = now - Duration::hours(1);
        repo.create(&expired_session).await.unwrap();

        let valid_session = test_session(user_id);
        repo.create(&valid_session).await.unwrap();

        let deleted = repo.delete_expired().await.unwrap();
        assert_eq!(deleted, 1);

        let sessions = repo.find_by_user_id(user_id).await.unwrap();
        assert_eq!(sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_account_operations() {
        let repo = InMemoryAccountRepository::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let account = Account {
            id: Uuid::new_v4(),
            user_id,
            provider: "google".to_string(),
            provider_account_id: "12345".to_string(),
            access_token: Some("token".to_string()),
            refresh_token: None,
            expires_at: None,
            created_at: now,
        };

        repo.create(&account).await.unwrap();

        let found = repo.find_by_provider("google", "12345").await.unwrap();
        assert!(found.is_some());

        let user_accounts = repo.find_by_user_id(user_id).await.unwrap();
        assert_eq!(user_accounts.len(), 1);

        repo.delete(account.id).await.unwrap();
        let not_found = repo.find_by_provider("google", "12345").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_verification_token_operations() {
        let repo = InMemoryVerificationTokenRepository::new();
        let now = Utc::now();

        let token = VerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            token_type: TokenType::EmailVerification,
            expires_at: now + Duration::hours(1),
            created_at: now,
        };

        repo.create(&token).await.unwrap();

        let found = repo.find_by_token_hash("hash123").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().token_type, TokenType::EmailVerification);
    }

    #[tokio::test]
    async fn test_verification_token_delete_expired() {
        let repo = InMemoryVerificationTokenRepository::new();
        let now = Utc::now();

        let expired = VerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "expired".to_string(),
            token_type: TokenType::PasswordReset,
            expires_at: now - Duration::hours(1),
            created_at: now - Duration::hours(2),
        };
        repo.create(&expired).await.unwrap();

        let valid = VerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "valid".to_string(),
            token_type: TokenType::MagicLink,
            expires_at: now + Duration::hours(1),
            created_at: now,
        };
        repo.create(&valid).await.unwrap();

        let deleted = repo.delete_expired().await.unwrap();
        assert_eq!(deleted, 1);

        assert!(repo.find_by_token_hash("valid").await.unwrap().is_some());
        assert!(repo.find_by_token_hash("expired").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_two_factor_operations() {
        let repo = InMemoryTwoFactorRepository::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let tf = TwoFactor {
            id: Uuid::new_v4(),
            user_id,
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec!["code1".to_string(), "code2".to_string()],
            enabled: false,
            verified: false,
            created_at: now,
            updated_at: now,
        };

        repo.create(&tf).await.unwrap();

        let found = repo.find_by_user_id(user_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().backup_codes.len(), 2);

        repo.delete_by_user_id(user_id).await.unwrap();
        assert!(repo.find_by_user_id(user_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_organization_create() {
        let repo = InMemoryOrganizationRepository::new();
        let org = test_organization();

        let created = repo.create(&org).await.unwrap();
        assert_eq!(created.name, org.name);
    }

    #[tokio::test]
    async fn test_organization_duplicate_slug() {
        let repo = InMemoryOrganizationRepository::new();
        let org1 = test_organization();
        let mut org2 = test_organization();
        org2.slug = org1.slug.clone();

        repo.create(&org1).await.unwrap();
        let result = repo.create(&org2).await;

        assert!(matches!(result, Err(TsaError::OrganizationAlreadyExists)));
    }

    #[tokio::test]
    async fn test_organization_find_by_slug() {
        let repo = InMemoryOrganizationRepository::new();
        let org = test_organization();
        repo.create(&org).await.unwrap();

        let found = repo.find_by_slug(&org.slug).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, org.id);
    }

    #[tokio::test]
    async fn test_organization_member_operations() {
        let repo = InMemoryOrganizationMemberRepository::new();
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let member = OrganizationMember {
            id: Uuid::new_v4(),
            organization_id: org_id,
            user_id,
            role: OrganizationRole::Admin,
            created_at: now,
            updated_at: now,
        };

        repo.create(&member).await.unwrap();

        let found = repo.find_by_org_and_user(org_id, user_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().role, OrganizationRole::Admin);

        let org_members = repo.find_by_organization(org_id).await.unwrap();
        assert_eq!(org_members.len(), 1);

        let user_orgs = repo.find_by_user(user_id).await.unwrap();
        assert_eq!(user_orgs.len(), 1);
    }

    #[tokio::test]
    async fn test_organization_member_duplicate() {
        let repo = InMemoryOrganizationMemberRepository::new();
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let member1 = OrganizationMember {
            id: Uuid::new_v4(),
            organization_id: org_id,
            user_id,
            role: OrganizationRole::Member,
            created_at: now,
            updated_at: now,
        };

        let member2 = OrganizationMember {
            id: Uuid::new_v4(),
            organization_id: org_id,
            user_id,
            role: OrganizationRole::Admin,
            created_at: now,
            updated_at: now,
        };

        repo.create(&member1).await.unwrap();
        let result = repo.create(&member2).await;

        assert!(matches!(result, Err(TsaError::AlreadyOrganizationMember)));
    }

    #[tokio::test]
    async fn test_organization_invitation_operations() {
        let repo = InMemoryOrganizationInvitationRepository::new();
        let org_id = Uuid::new_v4();
        let now = Utc::now();

        let invitation = OrganizationInvitation {
            id: Uuid::new_v4(),
            organization_id: org_id,
            email: "invite@test.com".to_string(),
            role: OrganizationRole::Member,
            token_hash: "hash123".to_string(),
            invited_by: Uuid::new_v4(),
            status: InvitationStatus::Pending,
            expires_at: now + Duration::days(7),
            created_at: now,
        };

        repo.create(&invitation).await.unwrap();

        let found = repo.find_by_token_hash("hash123").await.unwrap();
        assert!(found.is_some());

        let pending = repo
            .find_pending_by_org_and_email(org_id, "invite@test.com")
            .await
            .unwrap();
        assert!(pending.is_some());

        repo.update_status(invitation.id, InvitationStatus::Accepted)
            .await
            .unwrap();
        let updated = repo.find_by_id(invitation.id).await.unwrap().unwrap();
        assert_eq!(updated.status, InvitationStatus::Accepted);
    }

    #[tokio::test]
    async fn test_api_key_operations() {
        let repo = InMemoryApiKeyRepository::new();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let now = Utc::now();

        let api_key = ApiKey {
            id: Uuid::new_v4(),
            user_id,
            organization_id: Some(org_id),
            name: "Test Key".to_string(),
            key_hash: "keyhash123".to_string(),
            prefix: "tsa_abc".to_string(),
            scopes: vec!["read:users".to_string()],
            expires_at: None,
            last_used_at: None,
            created_at: now,
        };

        repo.create(&api_key).await.unwrap();

        let found = repo.find_by_key_hash("keyhash123").await.unwrap();
        assert!(found.is_some());

        let found_prefix = repo.find_by_prefix("tsa_abc").await.unwrap();
        assert!(found_prefix.is_some());

        let user_keys = repo.find_by_user(user_id).await.unwrap();
        assert_eq!(user_keys.len(), 1);

        let org_keys = repo.find_by_organization(org_id).await.unwrap();
        assert_eq!(org_keys.len(), 1);

        repo.update_last_used(api_key.id).await.unwrap();
        let updated = repo.find_by_id(api_key.id).await.unwrap().unwrap();
        assert!(updated.last_used_at.is_some());
    }

    #[tokio::test]
    async fn test_passkey_operations() {
        let repo = InMemoryPasskeyRepository::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let passkey = Passkey {
            id: Uuid::new_v4(),
            user_id,
            credential_id: vec![1, 2, 3, 4],
            public_key: vec![5, 6, 7, 8],
            counter: 0,
            name: "My Passkey".to_string(),
            transports: Some(vec!["usb".to_string()]),
            created_at: now,
            last_used_at: None,
        };

        repo.create(&passkey).await.unwrap();

        let found = repo.find_by_credential_id(&[1, 2, 3, 4]).await.unwrap();
        assert!(found.is_some());

        let user_passkeys = repo.find_by_user(user_id).await.unwrap();
        assert_eq!(user_passkeys.len(), 1);
    }

    #[tokio::test]
    async fn test_passkey_duplicate_credential() {
        let repo = InMemoryPasskeyRepository::new();
        let now = Utc::now();

        let passkey1 = Passkey {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            credential_id: vec![1, 2, 3],
            public_key: vec![4, 5, 6],
            counter: 0,
            name: "Key 1".to_string(),
            transports: None,
            created_at: now,
            last_used_at: None,
        };

        let passkey2 = Passkey {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            credential_id: vec![1, 2, 3],
            public_key: vec![7, 8, 9],
            counter: 0,
            name: "Key 2".to_string(),
            transports: None,
            created_at: now,
            last_used_at: None,
        };

        repo.create(&passkey1).await.unwrap();
        let result = repo.create(&passkey2).await;

        assert!(matches!(result, Err(TsaError::PasskeyAlreadyRegistered)));
    }

    #[tokio::test]
    async fn test_passkey_challenge_operations() {
        let repo = InMemoryPasskeyChallengeRepository::new();
        let now = Utc::now();

        let challenge = PasskeyChallenge {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            challenge: vec![1, 2, 3, 4, 5],
            challenge_type: PasskeyChallengeType::Registration,
            state: vec![10, 20, 30],
            expires_at: now + Duration::minutes(5),
            created_at: now,
        };

        repo.create(&challenge).await.unwrap();

        let found = repo.find_by_challenge(&[1, 2, 3, 4, 5]).await.unwrap();
        assert!(found.is_some());
        assert_eq!(
            found.unwrap().challenge_type,
            PasskeyChallengeType::Registration
        );
    }

    #[tokio::test]
    async fn test_passkey_challenge_delete_expired() {
        let repo = InMemoryPasskeyChallengeRepository::new();
        let now = Utc::now();

        let expired = PasskeyChallenge {
            id: Uuid::new_v4(),
            user_id: None,
            challenge: vec![1, 1, 1],
            challenge_type: PasskeyChallengeType::Authentication,
            state: vec![],
            expires_at: now - Duration::minutes(1),
            created_at: now - Duration::minutes(10),
        };
        repo.create(&expired).await.unwrap();

        let valid = PasskeyChallenge {
            id: Uuid::new_v4(),
            user_id: None,
            challenge: vec![2, 2, 2],
            challenge_type: PasskeyChallengeType::Registration,
            state: vec![],
            expires_at: now + Duration::minutes(5),
            created_at: now,
        };
        repo.create(&valid).await.unwrap();

        let deleted = repo.delete_expired().await.unwrap();
        assert_eq!(deleted, 1);

        assert!(repo.find_by_challenge(&[2, 2, 2]).await.unwrap().is_some());
        assert!(repo.find_by_challenge(&[1, 1, 1]).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_adapter_provides_all_repositories() {
        let adapter = InMemoryAdapter::new();

        let user = test_user();
        adapter.users().create(&user).await.unwrap();

        let session = test_session(user.id);
        adapter.sessions().create(&session).await.unwrap();

        let found_user = adapter.users().find_by_id(user.id).await.unwrap();
        assert!(found_user.is_some());

        let found_session = adapter.sessions().find_by_user_id(user.id).await.unwrap();
        assert_eq!(found_session.len(), 1);
    }
}
