use sqlx::{PgPool, migrate::MigrateError};

/// Applies pending migrations from ./migrations (embedded at compile time).
/// Already-applied migrations are skipped via SQLx's `_sqlx_migrations` table.
pub async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    tracing::info!("running database migrations");
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("database migrations up to date");
    Ok(())
}
