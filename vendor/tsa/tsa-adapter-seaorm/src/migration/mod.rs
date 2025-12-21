pub use sea_orm_migration::prelude::*;

mod m20241219_000001_create_users_table;
mod m20241219_000002_create_sessions_table;
mod m20241219_000003_create_accounts_table;
mod m20241219_000004_create_verification_tokens_table;
mod m20241219_000005_create_two_factors_table;
mod m20241219_000006_create_organizations_table;
mod m20241219_000007_create_organization_members_table;
mod m20241219_000008_create_organization_invitations_table;
mod m20241219_000009_create_api_keys_table;
mod m20241219_000010_create_passkeys_table;
mod m20241219_000011_create_passkey_challenges_table;
mod m20241219_000012_create_audit_logs_table;
mod m20241219_000013_create_account_lockouts_table;
mod m20241219_000014_create_password_history_table;
mod m20241219_000015_create_ip_rules_table;
mod m20241219_000016_create_impersonation_sessions_table;

pub use m20241219_000001_create_users_table::Users;
pub use m20241219_000002_create_sessions_table::Sessions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241219_000001_create_users_table::Migration),
            Box::new(m20241219_000002_create_sessions_table::Migration),
            Box::new(m20241219_000003_create_accounts_table::Migration),
            Box::new(m20241219_000004_create_verification_tokens_table::Migration),
            Box::new(m20241219_000005_create_two_factors_table::Migration),
            Box::new(m20241219_000006_create_organizations_table::Migration),
            Box::new(m20241219_000007_create_organization_members_table::Migration),
            Box::new(m20241219_000008_create_organization_invitations_table::Migration),
            Box::new(m20241219_000009_create_api_keys_table::Migration),
            Box::new(m20241219_000010_create_passkeys_table::Migration),
            Box::new(m20241219_000011_create_passkey_challenges_table::Migration),
            Box::new(m20241219_000012_create_audit_logs_table::Migration),
            Box::new(m20241219_000013_create_account_lockouts_table::Migration),
            Box::new(m20241219_000014_create_password_history_table::Migration),
            Box::new(m20241219_000015_create_ip_rules_table::Migration),
            Box::new(m20241219_000016_create_impersonation_sessions_table::Migration),
        ]
    }
}
