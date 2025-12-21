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
                    .table(OrganizationMembers::Table)
                    .if_not_exists()
                    .col(uuid(OrganizationMembers::Id).primary_key())
                    .col(uuid(OrganizationMembers::OrganizationId).not_null())
                    .col(uuid(OrganizationMembers::UserId).not_null())
                    .col(string(OrganizationMembers::Role).not_null())
                    .col(timestamp_with_time_zone(OrganizationMembers::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(OrganizationMembers::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_organization_members_org")
                            .from(
                                OrganizationMembers::Table,
                                OrganizationMembers::OrganizationId,
                            )
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_organization_members_user")
                            .from(OrganizationMembers::Table, OrganizationMembers::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_members_org_user")
                    .table(OrganizationMembers::Table)
                    .col(OrganizationMembers::OrganizationId)
                    .col(OrganizationMembers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_members_user")
                    .table(OrganizationMembers::Table)
                    .col(OrganizationMembers::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrganizationMembers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum OrganizationMembers {
    Table,
    Id,
    OrganizationId,
    UserId,
    Role,
    CreatedAt,
    UpdatedAt,
}
