use sea_orm_migration::{prelude::*, schema::*};

use super::m20241219_000001_create_users_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccountLockouts::Table)
                    .if_not_exists()
                    .col(uuid(AccountLockouts::Id).primary_key())
                    .col(uuid(AccountLockouts::UserId).unique_key().not_null())
                    .col(integer(AccountLockouts::FailedAttempts).not_null())
                    .col(timestamp_with_time_zone_null(AccountLockouts::LockedUntil))
                    .col(timestamp_with_time_zone(AccountLockouts::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(AccountLockouts::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_account_lockouts_user")
                            .from(AccountLockouts::Table, AccountLockouts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_account_lockouts_user")
                    .table(AccountLockouts::Table)
                    .col(AccountLockouts::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccountLockouts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AccountLockouts {
    Table,
    Id,
    UserId,
    FailedAttempts,
    LockedUntil,
    CreatedAt,
    UpdatedAt,
}
