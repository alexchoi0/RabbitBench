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
                    .table(OrganizationInvitations::Table)
                    .if_not_exists()
                    .col(uuid(OrganizationInvitations::Id).primary_key())
                    .col(uuid(OrganizationInvitations::OrganizationId).not_null())
                    .col(string(OrganizationInvitations::Email).not_null())
                    .col(string(OrganizationInvitations::Role).not_null())
                    .col(
                        string(OrganizationInvitations::TokenHash)
                            .unique_key()
                            .not_null(),
                    )
                    .col(uuid(OrganizationInvitations::InvitedBy).not_null())
                    .col(string(OrganizationInvitations::Status).not_null())
                    .col(timestamp_with_time_zone(OrganizationInvitations::ExpiresAt).not_null())
                    .col(timestamp_with_time_zone(OrganizationInvitations::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_organization_invitations_org")
                            .from(
                                OrganizationInvitations::Table,
                                OrganizationInvitations::OrganizationId,
                            )
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_organization_invitations_inviter")
                            .from(
                                OrganizationInvitations::Table,
                                OrganizationInvitations::InvitedBy,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_invitations_org")
                    .table(OrganizationInvitations::Table)
                    .col(OrganizationInvitations::OrganizationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_invitations_email")
                    .table(OrganizationInvitations::Table)
                    .col(OrganizationInvitations::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_invitations_token")
                    .table(OrganizationInvitations::Table)
                    .col(OrganizationInvitations::TokenHash)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(OrganizationInvitations::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum OrganizationInvitations {
    Table,
    Id,
    OrganizationId,
    Email,
    Role,
    TokenHash,
    InvitedBy,
    Status,
    ExpiresAt,
    CreatedAt,
}
