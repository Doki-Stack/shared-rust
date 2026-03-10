# shared-rust

Shared Rust crate for the Doki Stack platform. Provides foundational types and utilities used by all Rust-based microservices.

## Purpose

This crate contains the common patterns that every Rust service in the Doki Stack platform depends on. It enforces consistency for error handling, tracing, observability, and request processing.

## Technology Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (2021 edition) |
| Web Framework | axum 0.7 |
| Async Runtime | tokio |
| Database | sqlx (PostgreSQL) |
| Tracing | tracing + tracing-subscriber |
| Observability | opentelemetry + opentelemetry-otlp |
| Error Handling | thiserror |
| Serialization | serde + serde_json |

## What's Included

- **Error Types** — Platform-wide error enum using `thiserror`, maps to standard error envelope JSON
- **Tracing Setup** — Initializes `tracing-subscriber` with OpenTelemetry exporter to Tempo
- **OTel Init** — TracerProvider and MeterProvider for traces and metrics
- **org_id Extractor** — Axum extractor that pulls `org_id` from headers and validates it
- **sqlx Helpers** — Connection pool factory, health check query, common query patterns
- **Config** — Environment variable loader with validation

## Crate

```toml
[dependencies]
doki-shared = { git = "https://github.com/Doki-Stack/shared-rust" }
```

## Implementation Phase

**Phase 0** — Foundation. Built first as `mcp-scanner` and `mcp-execution` depend on it.

## License

Apache License 2.0 — see [LICENSE](LICENSE)
