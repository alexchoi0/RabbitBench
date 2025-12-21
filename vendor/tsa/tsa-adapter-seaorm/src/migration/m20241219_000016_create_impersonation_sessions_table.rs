use sea_orm_migration::{prelude::*, schema::*};

use super::m20241219_000001_create_users_table::Users;
use super::m20241219_000002_create_sessions_table::Sessions;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ImpersonationSessions::Table)
                    .if_not_exists()
                    .col(uuid(ImpersonationSessions::Id).primary_key())
                    .col(uuid(ImpersonationSessions::AdminId).not_null())
                    .col(uuid(ImpersonationSessions::TargetUserId).not_null())
                    .col(uuid(ImpersonationSessions::OriginalSessionId).not_null())
                    .col(
                        uuid(ImpersonationSessions::ImpersonationSessionId)
                            .unique_key()
                            .not_null(),
                    )
                    .col(string_null(ImpersonationSessions::Reason))
                    .col(timestamp_with_time_zone(ImpersonationSessions::StartedAt).not_null())
                    .col(timestamp_with_time_zone_null(
                        ImpersonationSessions::EndedAt,
                    ))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_imp_sessions_admin")
                            .from(ImpersonationSessions::Table, ImpersonationSessions::AdminId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_imp_sessions_target")
                            .from(
                                ImpersonationSessions::Table,
                                ImpersonationSessions::TargetUserId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_imp_sessions_orig")
                            .from(
                                ImpersonationSessions::Table,
                                ImpersonationSessions::OriginalSessionId,
                            )
                            .to(Sessions::Table, Sessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_imp_sessions_imp")
                            .from(
                                ImpersonationSessions::Table,
                                ImpersonationSessions::ImpersonationSessionId,
                            )
                            .to(Sessions::Table, Sessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_imp_sessions_admin")
                    .table(ImpersonationSessions::Table)
                    .col(ImpersonationSessions::AdminId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_imp_sessions_target")
                    .table(ImpersonationSessions::Table)
                    .col(ImpersonationSessions::TargetUserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_imp_sessions_session")
                    .table(ImpersonationSessions::Table)
                    .col(ImpersonationSessions::ImpersonationSessionId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ImpersonationSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ImpersonationSessions {
    Table,
    Id,
    AdminId,
    TargetUserId,
    OriginalSessionId,
    ImpersonationSessionId,
    Reason,
    StartedAt,
    EndedAt,
}
