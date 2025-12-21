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
                    .table(PasswordHistory::Table)
                    .if_not_exists()
                    .col(uuid(PasswordHistory::Id).primary_key())
                    .col(uuid(PasswordHistory::UserId).not_null())
                    .col(string(PasswordHistory::PasswordHash).not_null())
                    .col(timestamp_with_time_zone(PasswordHistory::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_password_history_user")
                            .from(PasswordHistory::Table, PasswordHistory::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_password_history_user")
                    .table(PasswordHistory::Table)
                    .col(PasswordHistory::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_password_history_created_at")
                    .table(PasswordHistory::Table)
                    .col(PasswordHistory::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PasswordHistory::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PasswordHistory {
    Table,
    Id,
    UserId,
    PasswordHash,
    CreatedAt,
}
