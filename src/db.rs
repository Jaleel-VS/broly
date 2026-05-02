use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

/// Build a Postgres connection pool. Sensible defaults for a Discord bot:
/// small pool, short acquire timeout so we fail fast on DB outages.
pub async fn connect(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .context("connecting to Postgres")
}

/// Runs any pending migrations from ./migrations at startup.
///
/// Creating a new migration:
///   sqlx migrate add -r <name>    (creates up/down SQL files)
///
/// Currently unused: main.rs disables auto-migrations because Broly
/// reads from a shared database. Re-enable when Broly owns its schema.
#[allow(dead_code)]
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("running migrations")?;
    info!("migrations applied");
    Ok(())
}
