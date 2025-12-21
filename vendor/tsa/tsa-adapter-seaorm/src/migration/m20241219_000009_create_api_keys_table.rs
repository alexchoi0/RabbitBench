use sea_orm_migration::{prelude::*, schema::*};

use super::m20241219_000001_create_users_table::Users;
use super::m20241219_000006_create_organizations_table::Organizations;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiKeys::Table)
                    .if_not_exists()
                    .col(uuid(ApiKeys::Id).primary_key())
                    .col(uuid(ApiKeys::UserId).not_null())
                    .col(uuid_null(ApiKeys::OrganizationId))
                    .col(string(ApiKeys::Name).not_null())
                    .col(string(ApiKeys::KeyHash).unique_key().not_null())
                    .col(string(ApiKeys::Prefix).not_null())
                    .col(json_binary(ApiKeys::Scopes).not_null())
                    .col(timestamp_with_time_zone_null(ApiKeys::ExpiresAt))
                    .col(timestamp_with_time_zone_null(ApiKeys::LastUsedAt))
                    .col(timestamp_with_time_zone(ApiKeys::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_api_keys_user")
                            .from(ApiKeys::Table, ApiKeys::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_api_keys_org")
                            .from(ApiKeys::Table, ApiKeys::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_user")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_org")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::OrganizationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_key_hash")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::KeyHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_prefix")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::Prefix)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiKeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ApiKeys {
    Table,
    Id,
    UserId,
    OrganizationId,
    Name,
    KeyHash,
    Prefix,
    Scopes,
    ExpiresAt,
    LastUsedAt,
    CreatedAt,
}
