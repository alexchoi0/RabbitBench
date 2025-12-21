use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Organizations::Table)
                    .if_not_exists()
                    .col(uuid(Organizations::Id).primary_key())
                    .col(string(Organizations::Name).not_null())
                    .col(string(Organizations::Slug).unique_key().not_null())
                    .col(string_null(Organizations::Logo))
                    .col(json_binary_null(Organizations::Metadata))
                    .col(timestamp_with_time_zone(Organizations::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Organizations::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organizations_slug")
                    .table(Organizations::Table)
                    .col(Organizations::Slug)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Organizations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Organizations {
    Table,
    Id,
    Name,
    Slug,
    Logo,
    Metadata,
    CreatedAt,
    UpdatedAt,
}
