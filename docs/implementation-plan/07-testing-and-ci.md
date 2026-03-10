# shared-rust Implementation Plan — Testing and CI/CD

## 1. Testing Strategy

| Layer | Location | Tools |
|-------|----------|-------|
| Unit tests | `#[cfg(test)]` in each module | `cargo test` |
| Integration tests | `tests/*.rs` | `cargo test`, `tower::ServiceExt` for axum |
| Feature-gated tests | `#[cfg(feature = "sqlx-pg")]` | Run with `--features sqlx-pg` |

## 2. Per-Module Test Summary Table

| Module | Unit Tests | Integration Tests |
|--------|------------|-------------------|
| error | Error constructors, code_and_retryable, status_code, From impls | IntoResponse via axum |
| tracing | init_tracing (idempotency) | — |
| otel | OtelGuard drop | trace_id_from_context (with span) |
| extractors | — | Valid/missing/invalid X-Org-Id |
| models | Round-trip serialization | — |
| db | DbConfig::default | pool_from_config, health_check (needs DB) |

## 3. CI Workflow

```yaml
# .github/workflows/ci.yml (full)
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

      - name: Clippy (all features)
        run: cargo clippy --all-features -- -D warnings

      - name: Test (all features)
        run: cargo test --all-features

      - name: Build (all features)
        run: cargo build --all-features
```

## 4. Release Strategy

| Step | Action |
|------|--------|
| Version bump | Update `Cargo.toml` version |
| Tag | `git tag shared-rust-v0.1.0` |
| Push | `git push origin shared-rust-v0.1.0` |
| Consumer dep | `doki-shared = { git = "...", tag = "shared-rust-v0.1.0" }` |

SemVer: 0.1.x for initial; 1.0.0 when API stable.

## 5. Makefile (Complete)

```makefile
.PHONY: build build-all test lint fmt fmt-fix check doc clean

all: check

build:
	cargo build

build-all:
	cargo build --all-features

test:
	cargo test --all-features

lint:
	cargo clippy --all-features -- -D warnings

fmt:
	cargo fmt --check

fmt-fix:
	cargo fmt

check: fmt lint test

doc:
	cargo doc --all-features --no-deps --open

clean:
	cargo clean
```

## 6. Quality Gates

| Gate | Command | Must Pass |
|------|---------|-----------|
| Format | `cargo fmt --check` | Yes |
| Clippy | `cargo clippy --all-features -- -D warnings` | Yes |
| Tests | `cargo test --all-features` | Yes |
| Build | `cargo build --all-features` | Yes |

## 7. Test Utilities

For axum handler tests, use `tower::ServiceExt::oneshot`:

```rust
use tower::ServiceExt;

let app = axum::Router::new().route("/", get(handler));
let response = app
    .oneshot(
        Request::builder()
            .uri("/")
            .header("X-Org-Id", uuid.to_string())
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
```

## 8. Feature Matrix for CI

| Features | Build | Test |
|----------|-------|------|
| default | ✓ | ✓ |
| otel | ✓ | ✓ |
| sqlx-pg | ✓ | ✓ (unit only; integration ignored without DB) |
| all-features | ✓ | ✓ |
