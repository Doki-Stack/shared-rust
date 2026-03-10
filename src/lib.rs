//! # doki-shared
//!
//! Foundational shared crate for Doki Stack Rust microservices.
//! Provides error handling, tracing, extractors, models, and optional db/otel.

pub mod config;
pub mod error;
pub mod extractors;
pub mod models;
pub mod tracing;

#[cfg(feature = "otel")]
pub mod otel;

#[cfg(feature = "sqlx-pg")]
pub mod db;

pub use error::{Error, ErrorCode, Result};
pub use extractors::OrgId;
pub use models::*;

#[cfg(feature = "otel")]
pub use otel::{init_otel, span_id_from_context, trace_id_from_context, OtelGuard};

#[cfg(not(feature = "otel"))]
pub fn trace_id_from_context() -> Option<String> {
    None
}

#[cfg(not(feature = "otel"))]
pub fn span_id_from_context() -> Option<String> {
    None
}

#[cfg(feature = "sqlx-pg")]
pub use db::{health_check, pool_from_config, DbConfig};
