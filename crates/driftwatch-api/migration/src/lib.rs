pub use sea_orm_migration::prelude::*;

mod m20241221_000001_create_driftwatch_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = tsa_adapter_seaorm::Migrator::migrations();
        migrations.push(Box::new(
            m20241221_000001_create_driftwatch_tables::Migration,
        ));
        migrations
    }
}
