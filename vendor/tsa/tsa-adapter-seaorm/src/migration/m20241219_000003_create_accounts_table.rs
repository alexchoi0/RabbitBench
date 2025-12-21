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
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(uuid(Accounts::Id).primary_key())
                    .col(uuid(Accounts::UserId).not_null())
                    .col(string(Accounts::Provider).not_null())
                    .col(string(Accounts::ProviderAccountId).not_null())
                    .col(text_null(Accounts::AccessToken))
                    .col(text_null(Accounts::RefreshToken))
                    .col(timestamp_with_time_zone_null(Accounts::ExpiresAt))
                    .col(timestamp_with_time_zone(Accounts::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_accounts_user_id")
                            .from(Accounts::Table, Accounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_accounts_user_id")
                    .table(Accounts::Table)
                    .col(Accounts::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_accounts_provider")
                    .table(Accounts::Table)
                    .col(Accounts::Provider)
                    .col(Accounts::ProviderAccountId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Accounts {
    Table,
    Id,
    UserId,
    Provider,
    ProviderAccountId,
    AccessToken,
    RefreshToken,
    ExpiresAt,
    CreatedAt,
}
