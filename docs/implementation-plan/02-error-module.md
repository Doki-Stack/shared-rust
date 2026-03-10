# shared-rust Implementation Plan — Error Module

## 1. error.rs — Full Implementation

```rust
//! Platform error type with ErrorCode enum and Axum IntoResponse.

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use tracing::Span;

use crate::models::ErrorEnvelope;

/// Platform result type alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Domain error codes for API responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Policy
    PolicyUnavailable,
    PolicyViolation,
    PolicyIngestInvalid,

    // Scanner
    ScannerCloneFailed,
    ScannerTimeout,
    ScannerAnalysisFailed,

    // Execution
    ExecutionPlanFailed,
    ExecutionVaultError,
    ExecutionStateLocked,
    ExecutionPlanExpired,
    ExecutionPlanNotApproved,

    // Auth
    Unauthorized,
    Forbidden,
    TokenExpired,

    // General
    InternalError,
    NotFound,
    BadRequest,
    RateLimited,

    // EE
    LicenseRequired,
    LicenseExpired,
}

/// Platform error type.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{message}")]
    Domain {
        code: ErrorCode,
        message: String,
        retryable: bool,
    },

    #[cfg(feature = "sqlx-pg")]
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl Error {
    /// Create a domain error.
    pub fn domain(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self::Domain {
            code,
            message: message.into(),
            retryable,
        }
    }

    /// 404 Not Found.
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::domain(
            ErrorCode::NotFound,
            format!("Resource not found: {}", resource.into()),
            false,
        )
    }

    /// 400 Bad Request.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::BadRequest, message, false)
    }

    /// 401 Unauthorized.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::Unauthorized, message, false)
    }

    /// 403 Forbidden.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::Forbidden, message, false)
    }

    /// 500 Internal Error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::InternalError, message, true)
    }

    /// 429 Rate Limited.
    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::RateLimited, message, true)
    }

    /// Extract error code and retryable flag.
    pub fn code_and_retryable(&self) -> (ErrorCode, bool) {
        match self {
            Self::Domain { code, retryable, .. } => (*code, *retryable),
            #[cfg(feature = "sqlx-pg")]
            Self::Sqlx(e) => {
                let retryable = matches!(
                    e,
                    sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed
                );
                (ErrorCode::InternalError, retryable)
            }
            Self::Anyhow(_) => (ErrorCode::InternalError, true),
        }
    }

    /// HTTP status code for this error.
    pub fn status_code(&self) -> axum::http::StatusCode {
        let (code, _) = self.code_and_retryable();
        match code {
            ErrorCode::BadRequest | ErrorCode::PolicyIngestInvalid => {
                axum::http::StatusCode::BAD_REQUEST
            }
            ErrorCode::Unauthorized | ErrorCode::TokenExpired => {
                axum::http::StatusCode::UNAUTHORIZED
            }
            ErrorCode::Forbidden
            | ErrorCode::PolicyViolation
            | ErrorCode::ExecutionPlanNotApproved
            | ErrorCode::LicenseRequired
            | ErrorCode::LicenseExpired => axum::http::StatusCode::FORBIDDEN,
            ErrorCode::NotFound => axum::http::StatusCode::NOT_FOUND,
            ErrorCode::RateLimited => axum::http::StatusCode::TOO_MANY_REQUESTS,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (code, retryable) = self.code_and_retryable();
        let message = self.to_string();
        let status = self.status_code();

        // Try to get trace_id from current span (if otel feature enabled)
        let trace_id = crate::trace_id_from_context();

        let envelope = ErrorEnvelope {
            error_code: format!("{:?}", code),
            message,
            trace_id,
            org_id: None, // Set by middleware if available
            retryable,
        };

        (status, Json(envelope)).into_response()
    }
}
```

## 2. ErrorEnvelope (models/envelope.rs)

```rust
/// Error response envelope for API consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub error_code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    pub retryable: bool,
}
```

Note: `trace_id_from_context()` is a stub when `otel` is disabled; returns `None`. See 03-tracing-and-otel.md.

## 3. From Implementations

Add to error.rs for common conversions:

```rust
impl From<uuid::Error> for Error {
    fn from(e: uuid::Error) -> Self {
        Self::bad_request(format!("Invalid UUID: {}", e))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::bad_request(format!("Invalid JSON: {}", e))
    }
}
```

## 4. Test Plan

| Test | Description |
|------|-------------|
| `test_error_constructors` | Verify `not_found`, `bad_request`, `unauthorized`, etc. produce correct ErrorCode |
| `test_code_and_retryable` | Domain errors return correct (code, retryable) |
| `test_status_code_mapping` | NotFound→404, BadRequest→400, InternalError→500 |
| `test_into_response` | Error.into_response() produces valid JSON envelope |
| `test_from_impls` | `anyhow::Error`.into() produces Error::Anyhow |
| `test_sqlx_error` | (behind sqlx-pg) sqlx::Error converts with retryable for pool errors |

### Test Snippets

```rust
// tests/error_test.rs
use doki_shared::{Error, ErrorCode};

#[test]
fn test_not_found() {
    let e = Error::not_found("task-123");
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::NotFound);
    assert!(!retryable);
}

#[test]
fn test_status_codes() {
    assert_eq!(Error::not_found("x").status_code().as_u16(), 404);
    assert_eq!(Error::bad_request("bad").status_code().as_u16(), 400);
}
```
