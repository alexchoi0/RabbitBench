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
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(uuid(AuditLogs::Id).primary_key())
                    .col(uuid_null(AuditLogs::UserId))
                    .col(uuid_null(AuditLogs::ActorId))
                    .col(string(AuditLogs::Action).not_null())
                    .col(string_null(AuditLogs::IpAddress))
                    .col(string_null(AuditLogs::UserAgent))
                    .col(string_null(AuditLogs::ResourceType))
                    .col(string_null(AuditLogs::ResourceId))
                    .col(json_binary_null(AuditLogs::Details))
                    .col(boolean(AuditLogs::Success).not_null())
                    .col(string_null(AuditLogs::ErrorMessage))
                    .col(timestamp_with_time_zone(AuditLogs::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_user")
                            .from(AuditLogs::Table, AuditLogs::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_actor")
                            .from(AuditLogs::Table, AuditLogs::ActorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_user")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_action")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::Action)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_created_at")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_ip")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::IpAddress)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AuditLogs {
    Table,
    Id,
    UserId,
    ActorId,
    Action,
    IpAddress,
    UserAgent,
    ResourceType,
    ResourceId,
    Details,
    Success,
    ErrorMessage,
    CreatedAt,
}
