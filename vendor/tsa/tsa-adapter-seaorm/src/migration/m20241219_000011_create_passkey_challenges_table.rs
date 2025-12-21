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
                    .table(PasskeyChallenges::Table)
                    .if_not_exists()
                    .col(uuid(PasskeyChallenges::Id).primary_key())
                    .col(uuid_null(PasskeyChallenges::UserId))
                    .col(binary(PasskeyChallenges::Challenge).unique_key().not_null())
                    .col(string(PasskeyChallenges::ChallengeType).not_null())
                    .col(binary(PasskeyChallenges::State).not_null())
                    .col(timestamp_with_time_zone(PasskeyChallenges::ExpiresAt).not_null())
                    .col(timestamp_with_time_zone(PasskeyChallenges::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_passkey_challenges_user")
                            .from(PasskeyChallenges::Table, PasskeyChallenges::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_passkey_challenges_user")
                    .table(PasskeyChallenges::Table)
                    .col(PasskeyChallenges::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_passkey_challenges_challenge")
                    .table(PasskeyChallenges::Table)
                    .col(PasskeyChallenges::Challenge)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_passkey_challenges_expires")
                    .table(PasskeyChallenges::Table)
                    .col(PasskeyChallenges::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PasskeyChallenges::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PasskeyChallenges {
    Table,
    Id,
    UserId,
    Challenge,
    ChallengeType,
    State,
    ExpiresAt,
    CreatedAt,
}
