use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

use crate::models::ErrorEnvelope;

pub type Result<T> = std::result::Result<T, Error>;

/// Domain error codes for API responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    PolicyUnavailable,
    PolicyViolation,
    PolicyIngestInvalid,

    ScannerCloneFailed,
    ScannerTimeout,
    ScannerAnalysisFailed,

    ExecutionPlanFailed,
    ExecutionVaultError,
    ExecutionStateLocked,
    ExecutionPlanExpired,
    ExecutionPlanNotApproved,

    Unauthorized,
    Forbidden,
    TokenExpired,

    InternalError,
    NotFound,
    BadRequest,
    RateLimited,

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
    pub fn domain(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self::Domain {
            code,
            message: message.into(),
            retryable,
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::domain(
            ErrorCode::NotFound,
            format!("Resource not found: {}", resource.into()),
            false,
        )
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::BadRequest, message, false)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::Unauthorized, message, false)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::Forbidden, message, false)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::InternalError, message, true)
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::domain(ErrorCode::RateLimited, message, true)
    }

    pub fn code_and_retryable(&self) -> (ErrorCode, bool) {
        match self {
            Self::Domain {
                code, retryable, ..
            } => (*code, *retryable),
            #[cfg(feature = "sqlx-pg")]
            Self::Sqlx(e) => {
                let retryable = matches!(e, sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed);
                (ErrorCode::InternalError, retryable)
            }
            Self::Anyhow(_) => (ErrorCode::InternalError, true),
        }
    }

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

        let trace_id = crate::trace_id_from_context();

        let envelope = ErrorEnvelope {
            error_code: format!("{:?}", code),
            message,
            trace_id,
            org_id: None,
            retryable,
        };

        (status, Json(envelope)).into_response()
    }
}

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
