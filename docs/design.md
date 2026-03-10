# shared-rust — High-Level Design

## Overview

`shared-rust` is the foundational Rust crate for the Doki Stack platform. All Rust microservices (`mcp-scanner`, `mcp-execution`, `ee-license-server`) depend on this crate for consistent error handling, tracing, and observability.

## Architecture

```
shared-rust/
├── src/
│   ├── lib.rs              # Re-exports all modules
│   ├── error.rs            # Platform error types (thiserror)
│   ├── tracing.rs          # Tracing + OTel initialization
│   ├── extractors.rs       # Axum extractors (OrgId, TraceContext)
│   ├── db.rs               # sqlx pool factory and helpers
│   ├── config.rs           # Environment variable loader
│   └── health.rs           # Health check endpoint handler
├── Cargo.toml
└── README.md
```

## Key Design Decisions

1. **thiserror for errors** — Derives `Display` and `Error` automatically. Each variant maps to an HTTP status code and error envelope.
2. **Feature flags** — Optional features for `axum`, `sqlx`, `qdrant` to keep the dependency tree small for consumers that don't need everything.
3. **org_id is mandatory** — The `OrgId` extractor returns `400 Bad Request` if `X-Org-Id` header is missing or empty.
4. **Tracing, not logging** — Uses the `tracing` crate ecosystem for structured, span-aware instrumentation.

## Consumers

| Service | Features Used |
|---------|--------------|
| `mcp-scanner` | All |
| `mcp-execution` | All |
| `ee-license-server` (EE) | error, tracing, config, db |
