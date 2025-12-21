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
                    .table(Passkeys::Table)
                    .if_not_exists()
                    .col(uuid(Passkeys::Id).primary_key())
                    .col(uuid(Passkeys::UserId).not_null())
                    .col(binary(Passkeys::CredentialId).unique_key().not_null())
                    .col(binary(Passkeys::PublicKey).not_null())
                    .col(big_integer(Passkeys::Counter).not_null())
                    .col(string(Passkeys::Name).not_null())
                    .col(json_binary_null(Passkeys::Transports))
                    .col(timestamp_with_time_zone(Passkeys::CreatedAt).not_null())
                    .col(timestamp_with_time_zone_null(Passkeys::LastUsedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_passkeys_user")
                            .from(Passkeys::Table, Passkeys::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_passkeys_user")
                    .table(Passkeys::Table)
                    .col(Passkeys::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_passkeys_credential_id")
                    .table(Passkeys::Table)
                    .col(Passkeys::CredentialId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Passkeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Passkeys {
    Table,
    Id,
    UserId,
    CredentialId,
    PublicKey,
    Counter,
    Name,
    Transports,
    CreatedAt,
    LastUsedAt,
}
