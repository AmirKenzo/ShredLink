use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{SqlitePool, migrate::Migrator};
use std::path::Path;
use std::str::FromStr;

use crate::config::Config;

pub type DbPool = SqlitePool;

pub async fn new_pool(database_url: &str) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(opts)
        .await?;
    Ok(pool)
}

pub async fn create_pool(config: &Config) -> Result<SqlitePool> {
    if config.database_url.starts_with("sqlite:") {
        if let Some(path) = config.database_url.strip_prefix("sqlite:") {
            let path = path.split('?').next().unwrap_or(path);
            if let Some(parent) = Path::new(path).parent() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
        }
    }
    new_pool(&config.database_url).await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migrations_dir = std::env::var("MIGRATIONS_DIR")
        .ok()
        .map(std::path::PathBuf::from)
        .filter(|p| p.exists())
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"));
    let migrator = Migrator::new(migrations_dir).await?;
    migrator.run(pool).await?;
    Ok(())
}
