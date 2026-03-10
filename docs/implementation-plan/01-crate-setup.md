# shared-rust Implementation Plan — Crate Setup

## 1. Cargo.toml

```toml
[package]
name = "doki-shared"
version = "0.1.0"
edition = "2021"
description = "Foundational shared crate for Doki Stack Rust microservices"
license = "Apache-2.0"
repository = "https://github.com/doki-stack/doki-stack"

[dependencies]
# Web framework
axum = { version = "0.7", features = ["json"] }
tokio = { version = "1", features = ["full"] }

# Error handling
thiserror = "1"
anyhow = "1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Identifiers
uuid = { version = "1", features = ["v4", "serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# HTTP utilities
tower = "0.4"
tower-http = { version = "0.5", features = ["trace"] }

# Optional: OpenTelemetry (behind feature)
opentelemetry = { version = "0.24", optional = true }
opentelemetry-otlp = { version = "0.17", optional = true }
opentelemetry-sdk = { version = "0.24", optional = true }
tracing-opentelemetry = { version = "0.24", optional = true }

# Optional: PostgreSQL (behind feature)
sqlx = {
    version = "0.8",
    optional = true,
    features = ["runtime-tokio", "postgres", "uuid", "chrono"]
}

[features]
default = []
otel = [
    "opentelemetry",
    "opentelemetry-otlp",
    "opentelemetry-sdk",
    "tracing-opentelemetry"
]
sqlx-pg = ["sqlx"]

[dev-dependencies]
tower = { version = "0.4", features = ["util"] }
tokio-test = "0.4"
```

## 2. lib.rs

```rust
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

// Re-exports for convenience
pub use error::{Error, ErrorCode, Result};
pub use extractors::OrgId;
pub use models::*;

#[cfg(feature = "otel")]
pub use otel::{init_otel, OtelGuard, trace_id_from_context, span_id_from_context};

#[cfg(not(feature = "otel"))]
pub fn trace_id_from_context() -> Option<String> {
    None
}
#[cfg(not(feature = "otel"))]
pub fn span_id_from_context() -> Option<String> {
    None
}

#[cfg(feature = "sqlx-pg")]
pub use db::{pool_from_config, health_check, DbConfig};
```

## 3. Makefile

```makefile
.PHONY: build test lint fmt check doc clean

# Default target
all: check

# Build (default features)
build:
	cargo build

# Build with all features
build-all:
	cargo build --all-features

# Run tests (all features)
test:
	cargo test --all-features

# Clippy (all features)
lint:
	cargo clippy --all-features -- -D warnings

# Format check
fmt:
	cargo fmt --check

# Format (fix)
fmt-fix:
	cargo fmt

# Full check: fmt + clippy + test
check: fmt lint test

# Generate docs
doc:
	cargo doc --all-features --no-deps --open

# Clean
clean:
	cargo clean
```

## 4. CI Workflow (.github/workflows/ci.yml)

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Test
        run: cargo test --all-features

      - name: Build
        run: cargo build --all-features
```

## 5. Versioning

| Stage | Version | Notes |
|-------|---------|-------|
| Initial | 0.1.0 | First usable release |
| Stable API | 1.0.0 | After Phase 1 consumers validated |

**SemVer rules:**
- Patch (0.1.x): Bug fixes, no API changes
- Minor (0.x.0): New features, backward-compatible
- Major (x.0.0): Breaking changes

**Consumer dependency:**
```toml
[dependencies]
doki-shared = { path = "../shared-rust" }  # Monorepo
# Or:
doki-shared = { git = "https://github.com/doki-stack/doki-stack", tag = "shared-rust-v0.1.0" }
```
