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
