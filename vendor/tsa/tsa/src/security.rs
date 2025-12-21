use chrono::{Duration, Utc};
use std::sync::Arc;
use tsa_core::{
    AccountLockout, AccountLockoutRepository, Adapter, AuditAction, AuditLog, AuditLogRepository,
    ImpersonationSession, ImpersonationSessionRepository, IpRule, IpRuleRepository, IpRuleType,
    PasswordHistory, PasswordHistoryRepository, PasswordPolicy, Result, TsaError,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub password_policy: PasswordPolicy,
    pub audit_retention_days: u32,
    pub ip_check_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
            password_policy: PasswordPolicy::default(),
            audit_retention_days: 90,
            ip_check_enabled: true,
        }
    }
}

impl SecurityConfig {
    pub fn strict() -> Self {
        Self {
            max_failed_attempts: 3,
            lockout_duration_minutes: 60,
            password_policy: PasswordPolicy::strict(),
            audit_retention_days: 365,
            ip_check_enabled: true,
        }
    }
}

pub struct SecurityManager<A: Adapter> {
    adapter: Arc<A>,
    config: SecurityConfig,
}

impl<A: Adapter> SecurityManager<A> {
    pub fn new(adapter: Arc<A>, config: SecurityConfig) -> Self {
        Self { adapter, config }
    }

    pub async fn log_audit(
        &self,
        action: AuditAction,
        user_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        resource_type: Option<String>,
        resource_id: Option<String>,
        details: Option<serde_json::Value>,
        success: bool,
        error_message: Option<String>,
    ) -> Result<AuditLog> {
        let log = AuditLog {
            id: Uuid::new_v4(),
            user_id,
            actor_id,
            action,
            ip_address,
            user_agent,
            resource_type,
            resource_id,
            details,
            success,
            error_message,
            created_at: Utc::now(),
        };

        self.adapter.audit_logs().create(&log).await
    }

    pub async fn check_account_locked(&self, user_id: Uuid) -> Result<bool> {
        if let Some(lockout) = self
            .adapter
            .account_lockouts()
            .find_by_user(user_id)
            .await?
        {
            if let Some(locked_until) = lockout.locked_until {
                if locked_until > Utc::now() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub async fn record_failed_attempt(&self, user_id: Uuid) -> Result<bool> {
        let lockout = self
            .adapter
            .account_lockouts()
            .increment_failed_attempts(user_id)
            .await?;

        if lockout.failed_attempts >= self.config.max_failed_attempts {
            let locked_until =
                Utc::now() + Duration::minutes(self.config.lockout_duration_minutes as i64);
            let mut updated = lockout;
            updated.locked_until = Some(locked_until);
            updated.updated_at = Utc::now();
            self.adapter.account_lockouts().update(&updated).await?;

            self.log_audit(
                AuditAction::AccountLocked,
                Some(user_id),
                None,
                None,
                None,
                Some("user".to_string()),
                Some(user_id.to_string()),
                Some(serde_json::json!({
                    "failed_attempts": updated.failed_attempts,
                    "locked_until": locked_until.to_rfc3339()
                })),
                true,
                None,
            )
            .await?;

            return Ok(true);
        }

        Ok(false)
    }

    pub async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<()> {
        self.adapter
            .account_lockouts()
            .reset_failed_attempts(user_id)
            .await
    }

    pub async fn unlock_account(&self, user_id: Uuid, actor_id: Option<Uuid>) -> Result<()> {
        self.adapter
            .account_lockouts()
            .reset_failed_attempts(user_id)
            .await?;

        self.log_audit(
            AuditAction::AccountUnlocked,
            Some(user_id),
            actor_id,
            None,
            None,
            Some("user".to_string()),
            Some(user_id.to_string()),
            None,
            true,
            None,
        )
        .await?;

        Ok(())
    }

    pub fn validate_password(&self, password: &str) -> Result<()> {
        match self.config.password_policy.validate(password) {
            Ok(()) => Ok(()),
            Err(errors) => Err(TsaError::PasswordPolicyViolation(errors.join("; "))),
        }
    }

    pub async fn check_password_history(
        &self,
        user_id: Uuid,
        new_password_hash: &str,
    ) -> Result<bool> {
        if self.config.password_policy.password_history_count == 0 {
            return Ok(true);
        }

        let history = self
            .adapter
            .password_history()
            .find_by_user(user_id, self.config.password_policy.password_history_count)
            .await?;

        for entry in history {
            if entry.password_hash == new_password_hash {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub async fn add_password_to_history(&self, user_id: Uuid, password_hash: &str) -> Result<()> {
        if self.config.password_policy.password_history_count == 0 {
            return Ok(());
        }

        let entry = PasswordHistory {
            id: Uuid::new_v4(),
            user_id,
            password_hash: password_hash.to_string(),
            created_at: Utc::now(),
        };

        self.adapter.password_history().create(&entry).await?;

        self.adapter
            .password_history()
            .delete_old_entries(user_id, self.config.password_policy.password_history_count)
            .await?;

        Ok(())
    }

    pub async fn check_ip_allowed(&self, ip_address: &str) -> Result<bool> {
        if !self.config.ip_check_enabled {
            return Ok(true);
        }

        let rules = self.adapter.ip_rules().find_active().await?;

        if rules.is_empty() {
            return Ok(true);
        }

        let mut has_allow_rules = false;
        let mut is_blocked = false;
        let mut is_allowed = false;

        for rule in &rules {
            if rule.rule_type == IpRuleType::Allow {
                has_allow_rules = true;
            }
            if rule.matches(ip_address) {
                match rule.rule_type {
                    IpRuleType::Block => is_blocked = true,
                    IpRuleType::Allow => is_allowed = true,
                }
            }
        }

        if is_blocked {
            return Ok(false);
        }

        if has_allow_rules && !is_allowed {
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn add_ip_rule(
        &self,
        ip_pattern: &str,
        rule_type: IpRuleType,
        description: Option<String>,
        expires_at: Option<chrono::DateTime<Utc>>,
        created_by: Option<Uuid>,
    ) -> Result<IpRule> {
        let rule = IpRule {
            id: Uuid::new_v4(),
            ip_pattern: ip_pattern.to_string(),
            rule_type,
            description,
            expires_at,
            created_by,
            created_at: Utc::now(),
        };

        let created = self.adapter.ip_rules().create(&rule).await?;

        let action = match rule_type {
            IpRuleType::Block => AuditAction::IpBlocked,
            IpRuleType::Allow => AuditAction::IpAllowed,
        };

        self.log_audit(
            action,
            None,
            created_by,
            None,
            None,
            Some("ip_rule".to_string()),
            Some(created.id.to_string()),
            Some(serde_json::json!({
                "pattern": ip_pattern,
                "rule_type": rule_type.to_string()
            })),
            true,
            None,
        )
        .await?;

        Ok(created)
    }

    pub async fn start_impersonation(
        &self,
        admin_id: Uuid,
        target_user_id: Uuid,
        original_session_id: Uuid,
        impersonation_session_id: Uuid,
        reason: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<ImpersonationSession> {
        let session = ImpersonationSession {
            id: Uuid::new_v4(),
            admin_id,
            target_user_id,
            original_session_id,
            impersonation_session_id,
            reason: reason.clone(),
            started_at: Utc::now(),
            ended_at: None,
        };

        let created = self
            .adapter
            .impersonation_sessions()
            .create(&session)
            .await?;

        self.log_audit(
            AuditAction::ImpersonationStarted,
            Some(target_user_id),
            Some(admin_id),
            ip_address,
            user_agent,
            Some("user".to_string()),
            Some(target_user_id.to_string()),
            Some(serde_json::json!({
                "reason": reason,
                "impersonation_id": created.id.to_string()
            })),
            true,
            None,
        )
        .await?;

        Ok(created)
    }

    pub async fn end_impersonation(
        &self,
        impersonation_session_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<ImpersonationSession> {
        let imp_session = self
            .adapter
            .impersonation_sessions()
            .find_by_session_id(impersonation_session_id)
            .await?
            .ok_or(TsaError::SessionNotFound)?;

        let ended = self
            .adapter
            .impersonation_sessions()
            .end_session(imp_session.id, Utc::now())
            .await?;

        self.log_audit(
            AuditAction::ImpersonationEnded,
            Some(ended.target_user_id),
            Some(ended.admin_id),
            ip_address,
            user_agent,
            Some("user".to_string()),
            Some(ended.target_user_id.to_string()),
            Some(serde_json::json!({
                "impersonation_id": ended.id.to_string(),
                "duration_seconds": (ended.ended_at.unwrap() - ended.started_at).num_seconds()
            })),
            true,
            None,
        )
        .await?;

        Ok(ended)
    }

    pub async fn is_impersonating(&self, session_id: Uuid) -> Result<Option<ImpersonationSession>> {
        self.adapter
            .impersonation_sessions()
            .find_by_session_id(session_id)
            .await
    }

    pub async fn get_audit_logs_for_user(
        &self,
        user_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<AuditLog>> {
        self.adapter
            .audit_logs()
            .find_by_user(user_id, limit, offset)
            .await
    }

    pub async fn get_recent_audit_logs(&self, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        self.adapter.audit_logs().find_recent(limit, offset).await
    }

    pub async fn get_failed_audit_logs(&self, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        self.adapter.audit_logs().find_failed(limit, offset).await
    }

    pub async fn cleanup_old_audit_logs(&self) -> Result<u64> {
        let before = Utc::now() - Duration::days(self.config.audit_retention_days as i64);
        self.adapter.audit_logs().delete_older_than(before).await
    }

    pub async fn get_ip_rules(&self) -> Result<Vec<IpRule>> {
        self.adapter.ip_rules().find_all().await
    }

    pub async fn delete_ip_rule(&self, id: Uuid) -> Result<()> {
        self.adapter.ip_rules().delete(id).await
    }

    pub async fn get_lockout_status(&self, user_id: Uuid) -> Result<Option<AccountLockout>> {
        self.adapter.account_lockouts().find_by_user(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsa_adapter::InMemoryAdapter;

    fn create_security_manager() -> SecurityManager<InMemoryAdapter> {
        let adapter = Arc::new(InMemoryAdapter::new());
        SecurityManager::new(adapter, SecurityConfig::default())
    }

    #[tokio::test]
    async fn test_password_validation() {
        let manager = create_security_manager();

        assert!(manager.validate_password("Password123").is_ok());
        assert!(manager.validate_password("short").is_err());
        assert!(manager.validate_password("nouppercase123").is_err());
        assert!(manager.validate_password("NOLOWERCASE123").is_err());
        assert!(manager.validate_password("NoNumbers").is_err());
    }

    #[tokio::test]
    async fn test_account_lockout() {
        let manager = create_security_manager();
        let user_id = Uuid::new_v4();

        assert!(!manager.check_account_locked(user_id).await.unwrap());

        for _ in 0..4 {
            let locked = manager.record_failed_attempt(user_id).await.unwrap();
            assert!(!locked);
        }

        let locked = manager.record_failed_attempt(user_id).await.unwrap();
        assert!(locked);

        assert!(manager.check_account_locked(user_id).await.unwrap());

        manager.unlock_account(user_id, None).await.unwrap();
        assert!(!manager.check_account_locked(user_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_ip_rules() {
        let manager = create_security_manager();

        assert!(manager.check_ip_allowed("192.168.1.1").await.unwrap());

        manager
            .add_ip_rule("192.168.1.100", IpRuleType::Block, None, None, None)
            .await
            .unwrap();

        assert!(!manager.check_ip_allowed("192.168.1.100").await.unwrap());
        assert!(manager.check_ip_allowed("192.168.1.1").await.unwrap());
    }

    #[tokio::test]
    async fn test_ip_cidr_matching() {
        let manager = create_security_manager();

        manager
            .add_ip_rule("10.0.0.0/8", IpRuleType::Block, None, None, None)
            .await
            .unwrap();

        assert!(!manager.check_ip_allowed("10.1.2.3").await.unwrap());
        assert!(!manager.check_ip_allowed("10.255.255.255").await.unwrap());
        assert!(manager.check_ip_allowed("192.168.1.1").await.unwrap());
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let manager = create_security_manager();
        let user_id = Uuid::new_v4();

        manager
            .log_audit(
                AuditAction::SigninSuccess,
                Some(user_id),
                None,
                Some("192.168.1.1".to_string()),
                Some("Mozilla/5.0".to_string()),
                None,
                None,
                None,
                true,
                None,
            )
            .await
            .unwrap();

        let logs = manager
            .get_audit_logs_for_user(user_id, 10, 0)
            .await
            .unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, AuditAction::SigninSuccess);
    }

    #[tokio::test]
    async fn test_impersonation() {
        let manager = create_security_manager();
        let admin_id = Uuid::new_v4();
        let target_user_id = Uuid::new_v4();
        let original_session_id = Uuid::new_v4();
        let imp_session_id = Uuid::new_v4();

        let imp = manager
            .start_impersonation(
                admin_id,
                target_user_id,
                original_session_id,
                imp_session_id,
                Some("Debugging user issue".to_string()),
                None,
                None,
            )
            .await
            .unwrap();

        assert!(imp.ended_at.is_none());

        let found = manager.is_impersonating(imp_session_id).await.unwrap();
        assert!(found.is_some());

        let ended = manager
            .end_impersonation(imp_session_id, None, None)
            .await
            .unwrap();
        assert!(ended.ended_at.is_some());
    }
}
