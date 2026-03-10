# shared-rust Implementation Plan — Extractors Module

## 1. extractors/org_id.rs

```rust
//! Axum extractor for X-Org-Id header.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::error::{Error, Result};

/// Extracted and validated organization ID from X-Org-Id header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrgId(pub Uuid);

const HEADER_NAME: &str = "x-org-id";

#[async_trait]
impl<S> FromRequestParts<S> for OrgId
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(HEADER_NAME)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error::bad_request("missing or invalid X-Org-Id header"))?;

        let uuid = Uuid::parse_str(value)
            .map_err(|e| Error::bad_request(format!("invalid org_id (expected UUID): {}", e)))?;

        Ok(OrgId(uuid))
    }
}

impl OrgId {
    /// Reference to the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Consume and return the inner UUID.
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}
```

## 2. extractors/mod.rs

```rust
//! Axum extractors for Doki Stack APIs.

pub mod org_id;

pub use org_id::OrgId;
```

## 3. Behavior

| Scenario | Result |
|----------|--------|
| Valid `X-Org-Id: 550e8400-e29b-41d4-a716-446655440000` | `OrgId(uuid)` |
| Missing header | `Error::bad_request("missing or invalid X-Org-Id header")` → 400 |
| Invalid UUID (e.g. `not-a-uuid`) | `Error::bad_request("invalid org_id (expected UUID): ...")` → 400 |
| Empty string | Treated as missing → 400 |

## 4. Usage in Handlers

```rust
use axum::{extract::Path, Json};
use uuid::Uuid;
use doki_shared::{OrgId, Result};

async fn get_task(
    OrgId(org_id): OrgId,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Task>> {
    // org_id is validated UUID
    let task = fetch_task(org_id, task_id).await?;
    Ok(Json(task))
}

async fn list_scans(OrgId(org_id): OrgId) -> Result<Json<Vec<Scan>>> {
    let scans = list_scans_for_org(org_id).await?;
    Ok(Json(scans))
}
```

## 5. Test Plan

| Test | Description |
|------|-------------|
| `test_valid_org_id` | Request with valid UUID header → OrgId extracted |
| `test_missing_header` | No X-Org-Id → 400, Error::bad_request |
| `test_invalid_uuid` | X-Org-Id: "abc" → 400 |
| `test_empty_string` | X-Org-Id: "" or whitespace → 400 |

### Test Snippets

```rust
// tests/extractors_test.rs
use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

use doki_shared::extractors::OrgId;

fn app() -> axum::Router {
    axum::Router::new().route("/", axum::routing::get(|OrgId(org_id): OrgId| async move {
        org_id.as_uuid().to_string()
    }))
}

#[tokio::test]
async fn test_valid_org_id() {
    let uuid = Uuid::new_v4();
    let req = Request::builder()
        .uri("/")
        .header("X-Org-Id", uuid.to_string())
        .body(Body::empty())
        .unwrap();

    let app = app();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_header() {
    let req = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let app = app();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_uuid() {
    let req = Request::builder()
        .uri("/")
        .header("X-Org-Id", "not-a-uuid")
        .body(Body::empty())
        .unwrap();

    let app = app();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```
