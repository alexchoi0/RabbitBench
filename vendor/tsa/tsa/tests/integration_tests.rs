use chrono::{Duration, Utc};
use tsa::{Auth, AuthCallbacks, AuthConfig, NoopCallbacks};
use tsa_adapter::InMemoryAdapter;
use tsa_core::{OrganizationRole, Result, TsaError, User};
use uuid::Uuid;

fn create_auth() -> Auth<InMemoryAdapter, NoopCallbacks> {
    Auth::new(InMemoryAdapter::new(), AuthConfig::default(), NoopCallbacks)
}

fn create_auth_with_verification() -> Auth<InMemoryAdapter, NoopCallbacks> {
    Auth::new(
        InMemoryAdapter::new(),
        AuthConfig::default().require_email_verification(true),
        NoopCallbacks,
    )
}

#[tokio::test]
async fn test_signup_creates_user_and_session() {
    let auth = create_auth();

    let (user, session, token) = auth
        .signup(
            "test@example.com",
            "password123",
            Some("Test User".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.name, Some("Test User".to_string()));
    assert!(!user.email_verified);
    assert!(!token.is_empty());
    assert_eq!(session.user_id, user.id);
}

#[tokio::test]
async fn test_signup_duplicate_email_fails() {
    let auth = create_auth();

    auth.signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let result = auth
        .signup("test@example.com", "different_password", None)
        .await;

    assert!(matches!(result, Err(TsaError::UserAlreadyExists)));
}

#[tokio::test]
async fn test_signin_with_valid_credentials() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (signin_user, session, token) = auth
        .signin("test@example.com", "password123", None, None)
        .await
        .unwrap();

    assert_eq!(signin_user.id, user.id);
    assert!(!token.is_empty());
    assert_eq!(session.user_id, user.id);
}

#[tokio::test]
async fn test_signin_with_invalid_password() {
    let auth = create_auth();

    auth.signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let result = auth
        .signin("test@example.com", "wrong_password", None, None)
        .await;

    assert!(matches!(result, Err(TsaError::InvalidCredentials)));
}

#[tokio::test]
async fn test_signin_nonexistent_user() {
    let auth = create_auth();

    let result = auth
        .signin("nonexistent@example.com", "password123", None, None)
        .await;

    assert!(matches!(result, Err(TsaError::InvalidCredentials)));
}

#[tokio::test]
async fn test_validate_session() {
    let auth = create_auth();

    let (user, _, token) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (validated_user, session) = auth.validate_session(&token).await.unwrap();

    assert_eq!(validated_user.id, user.id);
    assert_eq!(session.user_id, user.id);
}

#[tokio::test]
async fn test_validate_invalid_session() {
    let auth = create_auth();

    let result = auth.validate_session("invalid_token").await;

    assert!(matches!(result, Err(TsaError::SessionNotFound)));
}

#[tokio::test]
async fn test_signout_invalidates_session() {
    let auth = create_auth();

    let (_, _, token) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    auth.signout(&token).await.unwrap();

    let result = auth.validate_session(&token).await;
    assert!(matches!(result, Err(TsaError::SessionNotFound)));
}

#[tokio::test]
async fn test_refresh_session() {
    let auth = create_auth();

    let (_, original_session, token) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let refreshed_session = auth.refresh_session(&token).await.unwrap();

    assert!(refreshed_session.expires_at > original_session.expires_at);
}

#[tokio::test]
async fn test_get_user_sessions() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    auth.signin("test@example.com", "password123", None, None)
        .await
        .unwrap();
    auth.signin("test@example.com", "password123", None, None)
        .await
        .unwrap();

    let sessions = auth.get_user_sessions(user.id).await.unwrap();
    assert_eq!(sessions.len(), 3);
}

#[tokio::test]
async fn test_revoke_all_sessions() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    auth.signin("test@example.com", "password123", None, None)
        .await
        .unwrap();

    auth.revoke_all_sessions(user.id).await.unwrap();

    let sessions = auth.get_user_sessions(user.id).await.unwrap();
    assert!(sessions.is_empty());
}

#[tokio::test]
async fn test_revoke_other_sessions() {
    let auth = create_auth();

    let (user, current_session, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    auth.signin("test@example.com", "password123", None, None)
        .await
        .unwrap();
    auth.signin("test@example.com", "password123", None, None)
        .await
        .unwrap();

    auth.revoke_other_sessions(user.id, current_session.id)
        .await
        .unwrap();

    let sessions = auth.get_user_sessions(user.id).await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, current_session.id);
}

#[tokio::test]
async fn test_change_password() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "old_password", None)
        .await
        .unwrap();

    auth.change_password(user.id, "old_password", "new_password", false, None)
        .await
        .unwrap();

    let result = auth
        .signin("test@example.com", "old_password", None, None)
        .await;
    assert!(matches!(result, Err(TsaError::InvalidCredentials)));

    let (_, _, _) = auth
        .signin("test@example.com", "new_password", None, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_change_password_wrong_current() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let result = auth
        .change_password(user.id, "wrong_password", "new_password", false, None)
        .await;

    assert!(matches!(result, Err(TsaError::InvalidCredentials)));
}

#[tokio::test]
async fn test_create_organization() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (org, member) = auth
        .create_organization(user.id, "My Organization", "my-org")
        .await
        .unwrap();

    assert_eq!(org.name, "My Organization");
    assert_eq!(org.slug, "my-org");
    assert_eq!(member.user_id, user.id);
    assert_eq!(member.role, OrganizationRole::Owner);
}

#[tokio::test]
async fn test_create_organization_duplicate_slug() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    auth.create_organization(user.id, "Org 1", "my-org")
        .await
        .unwrap();

    let result = auth.create_organization(user.id, "Org 2", "my-org").await;

    assert!(matches!(result, Err(TsaError::OrganizationAlreadyExists)));
}

#[tokio::test]
async fn test_get_user_organizations() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    auth.create_organization(user.id, "Org 1", "org-1")
        .await
        .unwrap();
    auth.create_organization(user.id, "Org 2", "org-2")
        .await
        .unwrap();

    let orgs = auth.get_user_organizations(user.id).await.unwrap();

    assert_eq!(orgs.len(), 2);
}

#[tokio::test]
async fn test_add_organization_member() {
    let auth = create_auth();

    let (owner, _, _) = auth
        .signup("owner@example.com", "password123", None)
        .await
        .unwrap();

    let (member_user, _, _) = auth
        .signup("member@example.com", "password123", None)
        .await
        .unwrap();

    let (org, _) = auth
        .create_organization(owner.id, "My Org", "my-org")
        .await
        .unwrap();

    let member = auth
        .add_organization_member(owner.id, org.id, member_user.id, OrganizationRole::Member)
        .await
        .unwrap();

    assert_eq!(member.user_id, member_user.id);
    assert_eq!(member.role, OrganizationRole::Member);
}

#[tokio::test]
async fn test_update_member_role() {
    let auth = create_auth();

    let (owner, _, _) = auth
        .signup("owner@example.com", "password123", None)
        .await
        .unwrap();

    let (member_user, _, _) = auth
        .signup("member@example.com", "password123", None)
        .await
        .unwrap();

    let (org, _) = auth
        .create_organization(owner.id, "My Org", "my-org")
        .await
        .unwrap();

    auth.add_organization_member(owner.id, org.id, member_user.id, OrganizationRole::Member)
        .await
        .unwrap();

    let updated = auth
        .update_member_role(owner.id, org.id, member_user.id, OrganizationRole::Admin)
        .await
        .unwrap();

    assert_eq!(updated.role, OrganizationRole::Admin);
}

#[tokio::test]
async fn test_cannot_demote_last_owner() {
    let auth = create_auth();

    let (owner, _, _) = auth
        .signup("owner@example.com", "password123", None)
        .await
        .unwrap();

    let (org, _) = auth
        .create_organization(owner.id, "My Org", "my-org")
        .await
        .unwrap();

    let result = auth
        .update_member_role(owner.id, org.id, owner.id, OrganizationRole::Admin)
        .await;

    assert!(matches!(result, Err(TsaError::CannotRemoveLastOwner)));
}

#[tokio::test]
async fn test_remove_organization_member() {
    let auth = create_auth();

    let (owner, _, _) = auth
        .signup("owner@example.com", "password123", None)
        .await
        .unwrap();

    let (member_user, _, _) = auth
        .signup("member@example.com", "password123", None)
        .await
        .unwrap();

    let (org, _) = auth
        .create_organization(owner.id, "My Org", "my-org")
        .await
        .unwrap();

    auth.add_organization_member(owner.id, org.id, member_user.id, OrganizationRole::Member)
        .await
        .unwrap();

    auth.remove_organization_member(owner.id, org.id, member_user.id)
        .await
        .unwrap();

    let members = auth.get_organization_members(org.id).await.unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].0.id, owner.id);
}

#[tokio::test]
async fn test_create_api_key() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (api_key, full_key) = auth
        .create_api_key(
            user.id,
            "My API Key",
            vec!["read:users".to_string()],
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(api_key.name, "My API Key");
    assert!(full_key.starts_with("tsa_"));
    assert!(api_key.scopes.contains(&"read:users".to_string()));
}

#[tokio::test]
async fn test_validate_api_key() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (api_key, full_key) = auth
        .create_api_key(user.id, "My API Key", vec![], None, None)
        .await
        .unwrap();

    let (validated_key, validated_user) = auth.validate_api_key(&full_key).await.unwrap();

    assert_eq!(validated_user.id, user.id);
    assert_eq!(validated_key.id, api_key.id);
}

#[tokio::test]
async fn test_validate_invalid_api_key() {
    let auth = create_auth();

    let result = auth.validate_api_key("tsa_invalid_key").await;

    assert!(matches!(result, Err(TsaError::InvalidApiKey)));
}

#[tokio::test]
async fn test_list_api_keys() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    auth.create_api_key(user.id, "Key 1", vec![], None, None)
        .await
        .unwrap();
    auth.create_api_key(user.id, "Key 2", vec![], None, None)
        .await
        .unwrap();

    let keys = auth.list_api_keys(user.id).await.unwrap();

    assert_eq!(keys.len(), 2);
}

#[tokio::test]
async fn test_delete_api_key() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (api_key, _) = auth
        .create_api_key(user.id, "Key 1", vec![], None, None)
        .await
        .unwrap();

    auth.delete_api_key(user.id, api_key.id).await.unwrap();

    let keys = auth.list_api_keys(user.id).await.unwrap();
    assert!(keys.is_empty());
}

#[tokio::test]
async fn test_has_2fa_enabled() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let has_2fa = auth.has_2fa_enabled(user.id).await.unwrap();
    assert!(!has_2fa);
}

#[tokio::test]
async fn test_enable_2fa_returns_setup() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let setup = auth.enable_2fa(user.id).await.unwrap();

    assert!(!setup.secret.is_empty());
    assert!(setup.otpauth_url.contains("otpauth://"));
    assert!(!setup.backup_codes.is_empty());
}

#[tokio::test]
async fn test_session_stores_ip_and_user_agent() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (_, session, _) = auth
        .signin(
            "test@example.com",
            "password123",
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(session.ip_address, Some("192.168.1.1".to_string()));
    assert_eq!(session.user_agent, Some("Mozilla/5.0".to_string()));
}

#[tokio::test]
async fn test_get_organization_by_slug() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    auth.create_organization(user.id, "My Org", "my-org")
        .await
        .unwrap();

    let org = auth.get_organization_by_slug("my-org").await.unwrap();

    assert_eq!(org.name, "My Org");
}

#[tokio::test]
async fn test_organization_not_found() {
    let auth = create_auth();

    let result = auth.get_organization_by_slug("nonexistent").await;

    assert!(matches!(result, Err(TsaError::OrganizationNotFound)));
}

#[tokio::test]
async fn test_update_organization() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (org, _) = auth
        .create_organization(user.id, "My Org", "my-org")
        .await
        .unwrap();

    let updated = auth
        .update_organization(
            user.id,
            org.id,
            Some("Updated Name".to_string()),
            Some("https://logo.png".to_string()),
            None,
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.logo, Some("https://logo.png".to_string()));
}

#[tokio::test]
async fn test_delete_organization() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (org, _) = auth
        .create_organization(user.id, "My Org", "my-org")
        .await
        .unwrap();

    auth.delete_organization(user.id, org.id).await.unwrap();

    let result = auth.get_organization(org.id).await;
    assert!(matches!(result, Err(TsaError::OrganizationNotFound)));
}

#[tokio::test]
async fn test_member_cannot_delete_organization() {
    let auth = create_auth();

    let (owner, _, _) = auth
        .signup("owner@example.com", "password123", None)
        .await
        .unwrap();

    let (member_user, _, _) = auth
        .signup("member@example.com", "password123", None)
        .await
        .unwrap();

    let (org, _) = auth
        .create_organization(owner.id, "My Org", "my-org")
        .await
        .unwrap();

    auth.add_organization_member(owner.id, org.id, member_user.id, OrganizationRole::Member)
        .await
        .unwrap();

    let result = auth.delete_organization(member_user.id, org.id).await;

    assert!(matches!(result, Err(TsaError::InsufficientPermissions)));
}

#[tokio::test]
async fn test_config_builder() {
    let config = AuthConfig::new()
        .app_name("MyApp")
        .session_expiry(Duration::hours(1))
        .require_email_verification(true);

    assert_eq!(config.app_name, "MyApp");
    assert_eq!(config.session_expiry, Duration::hours(1));
    assert!(config.require_email_verification);
}

#[tokio::test]
async fn test_set_user_phone() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let updated = auth.set_user_phone(user.id, "+1234567890").await.unwrap();

    assert_eq!(updated.phone, Some("+1234567890".to_string()));
    assert!(!updated.phone_verified);
}

#[tokio::test]
async fn test_organization_api_keys() {
    let auth = create_auth();

    let (user, _, _) = auth
        .signup("test@example.com", "password123", None)
        .await
        .unwrap();

    let (org, _) = auth
        .create_organization(user.id, "My Org", "my-org")
        .await
        .unwrap();

    let (api_key, _) = auth
        .create_api_key(user.id, "Org Key", vec![], Some(org.id), None)
        .await
        .unwrap();

    assert_eq!(api_key.organization_id, Some(org.id));

    let org_keys = auth.list_organization_api_keys(org.id).await.unwrap();
    assert_eq!(org_keys.len(), 1);
}
