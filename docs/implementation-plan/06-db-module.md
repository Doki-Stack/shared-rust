# shared-rust Implementation Plan — DB Module

## 1. db/pool.rs (behind `sqlx-pg` feature)

```rust
//! PostgreSQL connection pool and health check.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::{Error, Result};

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Connection URL (e.g. postgres://user:pass@localhost/db).
    pub url: String,
    /// Maximum connections in the pool.
    pub max_connections: u32,
    /// Minimum idle connections.
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
///
/// Defaults: max_connections=10, min_connections=2, acquire_timeout=30s.
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
```

## 2. db/mod.rs

```rust
//! Database helpers for PostgreSQL.

mod pool;

pub use pool::{health_check, pool_from_config, DbConfig};
```

## 3. Defaults

| Setting | Value |
|---------|-------|
| max_connections | 10 |
| min_connections | 2 |
| acquire_timeout | 30s |

## 4. Usage

```rust
// In service main.rs
use doki_shared::{DbConfig, pool_from_config, health_check};

#[tokio::main]
async fn main() -> Result<()> {
    let config = DbConfig {
        url: std::env::var("DATABASE_URL").expect("DATABASE_URL required"),
        ..Default::default()
    };

    let pool = pool_from_config(&config).await?;

    // Health check endpoint
    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async move {
            health_check(&pool).await
                .map(|_| "ok")
                .map_err(|e| e.into_response())
        }));

    // ...
}
```

## 5. Test Plan

| Test | Description |
|------|-------------|
| `test_pool_from_config` | Requires test DB; pool creation succeeds |
| `test_health_check` | health_check on valid pool returns Ok(()) |
| `test_health_check_invalid` | health_check on invalid pool returns Err |
| `test_db_config_default` | DbConfig::default() has expected values |

### Test Snippets

```rust
// tests/db_test.rs (requires DATABASE_URL or skip when absent)
#[cfg(feature = "sqlx-pg")]
#[tokio::test]
#[ignore] // Run with: cargo test --features sqlx-pg -- --ignored
async fn test_pool_and_health_check() {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/postgres".to_string()
    });
    let config = DbConfig {
        url,
        ..Default::default()
    };
    let pool = pool_from_config(&config).await.unwrap();
    health_check(&pool).await.unwrap();
}
```
