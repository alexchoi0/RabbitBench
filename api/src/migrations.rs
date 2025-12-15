use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};

pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    tracing::info!("Running database migrations...");

    let migrations = vec![
        "ALTER TABLE branches ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW()",
        "ALTER TABLE testbeds ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW()",
        "ALTER TABLE measures ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW()",
        "ALTER TABLE benchmarks ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW()",
        "ALTER TABLE alerts ADD COLUMN IF NOT EXISTS current_value DOUBLE PRECISION",
        "ALTER TABLE alerts ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW()",
        r#"DO $$ BEGIN
            CREATE TYPE alert_status AS ENUM ('active', 'acknowledged', 'resolved');
        EXCEPTION WHEN duplicate_object THEN NULL;
        END $$"#,
        r#"DO $$
        BEGIN
            IF EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'alerts' AND column_name = 'status'
                AND data_type = 'character varying'
            ) THEN
                ALTER TABLE alerts ALTER COLUMN status DROP DEFAULT;
                ALTER TABLE alerts ALTER COLUMN status TYPE alert_status USING status::alert_status;
                ALTER TABLE alerts ALTER COLUMN status SET DEFAULT 'active'::alert_status;
            END IF;
        END $$"#,
    ];

    for sql in migrations {
        db.execute(Statement::from_string(
            db.get_database_backend(),
            sql.to_string(),
        ))
        .await?;
    }

    tracing::info!("Database migrations complete");
    Ok(())
}
