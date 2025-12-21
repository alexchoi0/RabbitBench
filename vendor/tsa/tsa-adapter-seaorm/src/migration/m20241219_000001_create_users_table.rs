use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(uuid(Users::Id).primary_key())
                    .col(string(Users::Email).unique_key().not_null())
                    .col(boolean(Users::EmailVerified).not_null().default(false))
                    .col(string_null(Users::Phone))
                    .col(boolean(Users::PhoneVerified).not_null().default(false))
                    .col(string_null(Users::Name))
                    .col(string_null(Users::Image))
                    .col(timestamp_with_time_zone(Users::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Users::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_phone")
                    .table(Users::Table)
                    .col(Users::Phone)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    Email,
    EmailVerified,
    Phone,
    PhoneVerified,
    Name,
    Image,
    CreatedAt,
    UpdatedAt,
}
