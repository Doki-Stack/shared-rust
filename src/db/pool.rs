use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::{Error, Result};

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            min_connections: 2,
        }
    }
}

/// Create a PgPool from configuration.
pub async fn pool_from_config(config: &DbConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&config.url)
        .await
        .map_err(|e| Error::internal(format!("Failed to connect to database: {}", e)))?;

    Ok(pool)
}

/// Health check: execute SELECT 1.
pub async fn health_check(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| Error::internal(format!("Database health check failed: {}", e)))?;
    Ok(())
}
