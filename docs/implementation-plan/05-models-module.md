# shared-rust Implementation Plan — Models Module

## 1. models/envelope.rs

```rust
//! Error and response envelopes.

use serde::{Deserialize, Serialize};

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

## 2. models/scanner.rs

```rust
//! Scanner (mcp-scanner) request/response types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Input for initiating a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanInput {
    pub repo_url: String,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
}

/// A file with context (e.g. Terraform file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    pub path: String,
    pub content_type: String,
    pub size_bytes: u64,
}

/// Result of a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: Uuid,
    pub status: ScanStatus,
    pub context_files: Vec<ContextFile>,
    pub terraform_files: Vec<String>,
}

/// Scan status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}
```

## 3. models/execution.rs

```rust
//! Execution (mcp-execution) request/response types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Input for Terraform plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanInput {
    pub scan_id: Uuid,
    pub workspace: String,
    pub variables: Option<serde_json::Value>,
}

/// A resource change from plan output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChange {
    pub address: String,
    pub action: String, // "create", "update", "delete", "no-op"
    pub change_type: Option<String>,
}

/// Output of Terraform plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOutput {
    pub plan_id: Uuid,
    pub status: ExecutionStatus,
    pub resource_changes: Vec<ResourceChange>,
    pub raw_output: Option<String>,
}

/// Input for Terraform apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyInput {
    pub plan_id: Uuid,
}

/// Result of Terraform apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub apply_id: Uuid,
    pub status: ExecutionStatus,
    pub output: Option<String>,
}

/// Execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}
```

## 4. models/common.rs

```rust
//! Common request/response types.

use serde::{Deserialize, Serialize};

/// Pagination parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

/// Generic API response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

/// Audit action for audit logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
}
```

## 5. models/mod.rs

```rust
//! Shared request/response types for Doki Stack APIs.

pub mod common;
pub mod envelope;
pub mod execution;
pub mod scanner;

pub use common::{ApiResponse, AuditAction, Pagination, PaginationMeta};
pub use envelope::ErrorEnvelope;
pub use execution::{ApplyInput, ApplyResult, ExecutionStatus, PlanInput, PlanOutput, ResourceChange};
pub use scanner::{ContextFile, ScanInput, ScanResult, ScanStatus};
```

## 6. Test Plan

| Test | Description |
|------|-------------|
| `test_error_envelope_roundtrip` | Serialize/deserialize ErrorEnvelope |
| `test_scan_input_roundtrip` | ScanInput JSON round-trip |
| `test_plan_output_roundtrip` | PlanOutput JSON round-trip |
| `test_pagination_default` | Pagination::default() values |
| `test_api_response_format` | ApiResponse<T> JSON structure |

### Test Snippets

```rust
// tests/models_test.rs
use doki_shared::models::*;
use serde_json;
use uuid::Uuid;

#[test]
fn test_error_envelope_roundtrip() {
    let envelope = ErrorEnvelope {
        error_code: "NOT_FOUND".to_string(),
        message: "Resource not found".to_string(),
        trace_id: Some("abc123".to_string()),
        org_id: Some(Uuid::nil().to_string()),
        retryable: false,
    };
    let json = serde_json::to_string(&envelope).unwrap();
    let decoded: ErrorEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.error_code, envelope.error_code);
}

#[test]
fn test_scan_input_roundtrip() {
    let input = ScanInput {
        repo_url: "https://github.com/org/repo".to_string(),
        branch: Some("main".to_string()),
        commit_sha: None,
    };
    let json = serde_json::to_value(&input).unwrap();
    assert_eq!(json["repo_url"], "https://github.com/org/repo");
}
```
