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
                    .table(IpRules::Table)
                    .if_not_exists()
                    .col(uuid(IpRules::Id).primary_key())
                    .col(string(IpRules::IpPattern).not_null())
                    .col(string(IpRules::RuleType).not_null())
                    .col(string_null(IpRules::Description))
                    .col(timestamp_with_time_zone_null(IpRules::ExpiresAt))
                    .col(uuid_null(IpRules::CreatedBy))
                    .col(timestamp_with_time_zone(IpRules::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ip_rules_creator")
                            .from(IpRules::Table, IpRules::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ip_rules_type")
                    .table(IpRules::Table)
                    .col(IpRules::RuleType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ip_rules_expires")
                    .table(IpRules::Table)
                    .col(IpRules::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IpRules::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum IpRules {
    Table,
    Id,
    IpPattern,
    RuleType,
    Description,
    ExpiresAt,
    CreatedBy,
    CreatedAt,
}
