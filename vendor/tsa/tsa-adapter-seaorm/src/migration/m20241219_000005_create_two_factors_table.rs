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
                    .table(TwoFactors::Table)
                    .if_not_exists()
                    .col(uuid(TwoFactors::Id).primary_key())
                    .col(uuid(TwoFactors::UserId).not_null().unique_key())
                    .col(string(TwoFactors::Secret).not_null())
                    .col(json_binary(TwoFactors::BackupCodes).not_null())
                    .col(boolean(TwoFactors::Enabled).not_null().default(false))
                    .col(boolean(TwoFactors::Verified).not_null().default(false))
                    .col(timestamp_with_time_zone(TwoFactors::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(TwoFactors::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_two_factors_user")
                            .from(TwoFactors::Table, TwoFactors::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_two_factors_user_id")
                    .table(TwoFactors::Table)
                    .col(TwoFactors::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TwoFactors::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TwoFactors {
    Table,
    Id,
    UserId,
    Secret,
    BackupCodes,
    Enabled,
    Verified,
    CreatedAt,
    UpdatedAt,
}
