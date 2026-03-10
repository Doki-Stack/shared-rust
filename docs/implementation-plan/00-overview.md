# shared-rust Implementation Plan — Overview

## 1. Crate Identity

| Attribute | Value |
|-----------|-------|
| **Crate name** | `doki-shared` |
| **Rust edition** | 2021 |
| **License** | Apache 2.0 |
| **Repository** | (Doki Stack monorepo) |

## 2. Purpose

`doki-shared` is the foundational Rust crate for the Doki Stack platform. It provides:

- **Error handling** — Platform-wide error type with `thiserror`, `ErrorKind` enum, and Axum `IntoResponse` impl
- **Observability** — `tracing-subscriber` initialization and optional OpenTelemetry (Tempo traces + Prometheus metrics)
- **Extractors** — Axum extractors for `X-Org-Id` header validation
- **Models** — Shared request/response types (serde) for scanner, execution, and common APIs
- **Database** — sqlx `PgPool` factory and health check helpers

**Phase 0 foundation.** It has **no dependency** on other Doki Stack repos.

## 3. Directory Structure

```
shared-rust/
├── Cargo.toml
├── Cargo.lock
├── Makefile
├── .github/workflows/ci.yml
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── tracing.rs
│   ├── otel.rs
│   ├── config.rs
│   ├── extractors/
│   │   ├── mod.rs
│   │   └── org_id.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── envelope.rs
│   │   ├── scanner.rs
│   │   ├── execution.rs
│   │   └── common.rs
│   └── db/
│       ├── mod.rs
│       └── pool.rs
├── tests/
│   ├── error_test.rs
│   ├── extractors_test.rs
│   └── models_test.rs
└── docs/
    └── implementation-plan/
        ├── 00-overview.md
        ├── 01-crate-setup.md
        ├── 02-error-module.md
        ├── 03-tracing-and-otel.md
        ├── 04-extractors-module.md
        ├── 05-models-module.md
        ├── 06-db-module.md
        └── 07-testing-and-ci.md
```

## 4. Module Summary Table

| Module | Purpose | Feature Gate | Dependencies |
|--------|---------|--------------|--------------|
| `error` | Platform error type, ErrorCode enum, IntoResponse | — | thiserror, anyhow, axum |
| `tracing` | tracing-subscriber JSON init | — | tracing, tracing-subscriber |
| `otel` | OpenTelemetry (Tempo + Prometheus) | `otel` | opentelemetry-* |
| `config` | Shared config types | — | serde |
| `extractors::org_id` | X-Org-Id header extractor | — | axum, uuid |
| `models` | Shared request/response types | — | serde, serde_json, uuid |
| `db` | PgPool factory, health check | `sqlx-pg` | sqlx |

## 5. Consumer Matrix

| Consumer Service | Phase | Uses |
|------------------|-------|------|
| **mcp-scanner** | CE Phase 1 | error, tracing, otel, extractors, models, db |
| **mcp-execution** | CE Phase 1 | error, tracing, otel, extractors, models, db |
| **ee-license-server** | EE Phase 3 | error, tracing, otel, extractors, models |

## 6. Phase Mapping

```
Phase 0 (Foundation)
    └── doki-shared (this crate)

Phase 1 (MCP + CE services)
    ├── mcp-scanner
    └── mcp-execution

Phase 3 (EE)
    └── ee-license-server
```

## 7. Implementation Order

1. **error** — Core error type; all other modules may depend on it
2. **config** — Minimal config types; used by db and services
3. **tracing** — Logging foundation; no external deps
4. **otel** — Optional observability; depends on tracing
5. **models** — Shared types; independent
6. **extractors** — Axum extractors; depends on error
7. **db** — Database helpers; depends on error, config

## 8. Effort Estimate

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Crate setup + CI | Cargo.toml, lib.rs, Makefile, CI | 0.5 day |
| error | Error type, ErrorCode, IntoResponse, tests | 0.5 day |
| config | Config structs | 0.25 day |
| tracing + otel | Init, OtelGuard, helpers | 0.75 day |
| models | envelope, scanner, execution, common | 1 day |
| extractors | OrgId extractor, tests | 0.5 day |
| db | Pool, health check, tests | 0.5 day |
| Polish | Docs, CI green, release prep | 0.5 day |
| **Total** | | **4–5 days** |

## 9. Feature Flags

| Feature | Purpose | Optional Dependencies |
|---------|---------|------------------------|
| `otel` | OpenTelemetry integration | opentelemetry, opentelemetry-otlp, opentelemetry-sdk, tracing-opentelemetry |
| `sqlx-pg` | PostgreSQL pool helpers | sqlx (runtime-tokio, postgres, uuid, chrono) |

Default features: none. Consumers enable only what they need.

## 10. Non-Negotiable Rules

1. **org_id everywhere** — X-Org-Id header mandatory, UUID format. All APIs, logs, and error envelopes include org_id.
2. **thiserror for errors** — All domain errors use `#[derive(Error)]` with proper `#[error(...)]` attributes.
3. **IntoResponse for Error** — All `Error` variants map to HTTP status codes and JSON error envelope.
4. **Structured JSON tracing** — `tracing-subscriber` with JSON formatter; no unstructured log lines.
5. **Error envelope format** — `{"error_code": "DOMAIN_CODE", "message": "...", "trace_id": "...", "org_id": "...", "retryable": false}`
6. **Secrets never in logs** — Redaction is service-local (e.g., mcp-execution); shared crate does not log secrets.
