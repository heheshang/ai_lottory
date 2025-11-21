use sqlx::{Pool, Sqlite, migrate::MigrateError, migrate::Migrator};

// Embed migrations - path relative to crate root
static MIGRATOR: Migrator = sqlx::migrate!("src/database/migrations");

pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), MigrateError> {
    MIGRATOR.run(pool).await
}