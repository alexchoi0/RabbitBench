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
                    .table(VerificationTokens::Table)
                    .if_not_exists()
                    .col(uuid(VerificationTokens::Id).primary_key())
                    .col(uuid(VerificationTokens::UserId).not_null())
                    .col(
                        string(VerificationTokens::TokenHash)
                            .unique_key()
                            .not_null(),
                    )
                    .col(string(VerificationTokens::TokenType).not_null())
                    .col(timestamp_with_time_zone(VerificationTokens::ExpiresAt).not_null())
                    .col(timestamp_with_time_zone(VerificationTokens::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_verification_tokens_user_id")
                            .from(VerificationTokens::Table, VerificationTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_verification_tokens_user_id")
                    .table(VerificationTokens::Table)
                    .col(VerificationTokens::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_verification_tokens_token_hash")
                    .table(VerificationTokens::Table)
                    .col(VerificationTokens::TokenHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_verification_tokens_expires_at")
                    .table(VerificationTokens::Table)
                    .col(VerificationTokens::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VerificationTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum VerificationTokens {
    Table,
    Id,
    UserId,
    TokenHash,
    TokenType,
    ExpiresAt,
    CreatedAt,
}
