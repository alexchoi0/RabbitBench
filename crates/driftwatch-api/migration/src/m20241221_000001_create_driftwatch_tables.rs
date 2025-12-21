use sea_orm_migration::{prelude::*, schema::*};
use tsa_adapter_seaorm::migration::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(uuid(Projects::Id).primary_key())
                    .col(uuid(Projects::UserId).not_null())
                    .col(string(Projects::Slug).not_null())
                    .col(string(Projects::Name).not_null())
                    .col(string_null(Projects::Description))
                    .col(boolean(Projects::Public).not_null().default(false))
                    .col(string_null(Projects::GithubRepo))
                    .col(string_null(Projects::GithubToken))
                    .col(
                        boolean(Projects::GithubPrComments)
                            .not_null()
                            .default(false),
                    )
                    .col(
                        boolean(Projects::GithubStatusChecks)
                            .not_null()
                            .default(false),
                    )
                    .col(timestamp_with_time_zone(Projects::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Projects::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Projects::Table, Projects::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_user_id")
                    .table(Projects::Table)
                    .col(Projects::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_user_slug")
                    .table(Projects::Table)
                    .col(Projects::UserId)
                    .col(Projects::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Branches::Table)
                    .if_not_exists()
                    .col(uuid(Branches::Id).primary_key())
                    .col(uuid(Branches::ProjectId).not_null())
                    .col(string(Branches::Name).not_null())
                    .col(timestamp_with_time_zone(Branches::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Branches::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Branches::Table, Branches::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_branches_project_name")
                    .table(Branches::Table)
                    .col(Branches::ProjectId)
                    .col(Branches::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Testbeds::Table)
                    .if_not_exists()
                    .col(uuid(Testbeds::Id).primary_key())
                    .col(uuid(Testbeds::ProjectId).not_null())
                    .col(string(Testbeds::Name).not_null())
                    .col(timestamp_with_time_zone(Testbeds::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Testbeds::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Testbeds::Table, Testbeds::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_testbeds_project_name")
                    .table(Testbeds::Table)
                    .col(Testbeds::ProjectId)
                    .col(Testbeds::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Measures::Table)
                    .if_not_exists()
                    .col(uuid(Measures::Id).primary_key())
                    .col(uuid(Measures::ProjectId).not_null())
                    .col(string(Measures::Name).not_null())
                    .col(string_null(Measures::Units))
                    .col(timestamp_with_time_zone(Measures::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Measures::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Measures::Table, Measures::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_measures_project_name")
                    .table(Measures::Table)
                    .col(Measures::ProjectId)
                    .col(Measures::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Benchmarks::Table)
                    .if_not_exists()
                    .col(uuid(Benchmarks::Id).primary_key())
                    .col(uuid(Benchmarks::ProjectId).not_null())
                    .col(string(Benchmarks::Name).not_null())
                    .col(timestamp_with_time_zone(Benchmarks::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Benchmarks::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Benchmarks::Table, Benchmarks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_benchmarks_project_name")
                    .table(Benchmarks::Table)
                    .col(Benchmarks::ProjectId)
                    .col(Benchmarks::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Reports::Table)
                    .if_not_exists()
                    .col(uuid(Reports::Id).primary_key())
                    .col(uuid(Reports::ProjectId).not_null())
                    .col(uuid(Reports::BranchId).not_null())
                    .col(uuid(Reports::TestbedId).not_null())
                    .col(string_null(Reports::GitHash))
                    .col(integer_null(Reports::PrNumber))
                    .col(timestamp_with_time_zone(Reports::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Reports::Table, Reports::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Reports::Table, Reports::BranchId)
                            .to(Branches::Table, Branches::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Reports::Table, Reports::TestbedId)
                            .to(Testbeds::Table, Testbeds::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_reports_project_id")
                    .table(Reports::Table)
                    .col(Reports::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_reports_branch_id")
                    .table(Reports::Table)
                    .col(Reports::BranchId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_reports_created_at")
                    .table(Reports::Table)
                    .col(Reports::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Metrics::Table)
                    .if_not_exists()
                    .col(uuid(Metrics::Id).primary_key())
                    .col(uuid(Metrics::ReportId).not_null())
                    .col(uuid(Metrics::BenchmarkId).not_null())
                    .col(uuid(Metrics::MeasureId).not_null())
                    .col(double(Metrics::Value).not_null())
                    .col(double_null(Metrics::LowerValue))
                    .col(double_null(Metrics::UpperValue))
                    .col(timestamp_with_time_zone(Metrics::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Metrics::Table, Metrics::ReportId)
                            .to(Reports::Table, Reports::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Metrics::Table, Metrics::BenchmarkId)
                            .to(Benchmarks::Table, Benchmarks::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Metrics::Table, Metrics::MeasureId)
                            .to(Measures::Table, Measures::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_metrics_report_id")
                    .table(Metrics::Table)
                    .col(Metrics::ReportId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_metrics_benchmark_id")
                    .table(Metrics::Table)
                    .col(Metrics::BenchmarkId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Thresholds::Table)
                    .if_not_exists()
                    .col(uuid(Thresholds::Id).primary_key())
                    .col(uuid(Thresholds::ProjectId).not_null())
                    .col(uuid_null(Thresholds::BranchId))
                    .col(uuid_null(Thresholds::TestbedId))
                    .col(uuid(Thresholds::MeasureId).not_null())
                    .col(double_null(Thresholds::UpperBoundary))
                    .col(double_null(Thresholds::LowerBoundary))
                    .col(integer(Thresholds::MinSampleSize).not_null().default(2))
                    .col(timestamp_with_time_zone(Thresholds::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Thresholds::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Thresholds::Table, Thresholds::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Thresholds::Table, Thresholds::BranchId)
                            .to(Branches::Table, Branches::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Thresholds::Table, Thresholds::TestbedId)
                            .to(Testbeds::Table, Testbeds::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Thresholds::Table, Thresholds::MeasureId)
                            .to(Measures::Table, Measures::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_thresholds_project_id")
                    .table(Thresholds::Table)
                    .col(Thresholds::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alerts::Table)
                    .if_not_exists()
                    .col(uuid(Alerts::Id).primary_key())
                    .col(uuid(Alerts::ThresholdId).not_null())
                    .col(uuid(Alerts::MetricId).not_null())
                    .col(double(Alerts::BaselineValue).not_null())
                    .col(double(Alerts::CurrentValue).not_null())
                    .col(double(Alerts::PercentChange).not_null())
                    .col(string(Alerts::Status).not_null().default("active"))
                    .col(timestamp_with_time_zone(Alerts::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(Alerts::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alerts::Table, Alerts::ThresholdId)
                            .to(Thresholds::Table, Thresholds::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alerts::Table, Alerts::MetricId)
                            .to(Metrics::Table, Metrics::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_alerts_threshold_id")
                    .table(Alerts::Table)
                    .col(Alerts::ThresholdId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_alerts_status")
                    .table(Alerts::Table)
                    .col(Alerts::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Flamegraphs::Table)
                    .if_not_exists()
                    .col(uuid(Flamegraphs::Id).primary_key())
                    .col(uuid(Flamegraphs::ReportId).not_null())
                    .col(uuid_null(Flamegraphs::BenchmarkId))
                    .col(string(Flamegraphs::StoragePath).not_null())
                    .col(string(Flamegraphs::FileName).not_null())
                    .col(integer(Flamegraphs::FileSize).not_null())
                    .col(timestamp_with_time_zone(Flamegraphs::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Flamegraphs::Table, Flamegraphs::ReportId)
                            .to(Reports::Table, Reports::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Flamegraphs::Table, Flamegraphs::BenchmarkId)
                            .to(Benchmarks::Table, Benchmarks::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_flamegraphs_report_id")
                    .table(Flamegraphs::Table)
                    .col(Flamegraphs::ReportId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Flamegraphs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Alerts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Thresholds::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Metrics::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Reports::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Benchmarks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Measures::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Testbeds::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Branches::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Projects {
    Table,
    Id,
    UserId,
    Slug,
    Name,
    Description,
    Public,
    GithubRepo,
    GithubToken,
    GithubPrComments,
    GithubStatusChecks,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Branches {
    Table,
    Id,
    ProjectId,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Testbeds {
    Table,
    Id,
    ProjectId,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Measures {
    Table,
    Id,
    ProjectId,
    Name,
    Units,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Benchmarks {
    Table,
    Id,
    ProjectId,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Reports {
    Table,
    Id,
    ProjectId,
    BranchId,
    TestbedId,
    GitHash,
    PrNumber,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum Metrics {
    Table,
    Id,
    ReportId,
    BenchmarkId,
    MeasureId,
    Value,
    LowerValue,
    UpperValue,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum Thresholds {
    Table,
    Id,
    ProjectId,
    BranchId,
    TestbedId,
    MeasureId,
    UpperBoundary,
    LowerBoundary,
    MinSampleSize,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Alerts {
    Table,
    Id,
    ThresholdId,
    MetricId,
    BaselineValue,
    CurrentValue,
    PercentChange,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Flamegraphs {
    Table,
    Id,
    ReportId,
    BenchmarkId,
    StoragePath,
    FileName,
    FileSize,
    CreatedAt,
}
