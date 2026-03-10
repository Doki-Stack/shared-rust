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
    pub action: String,
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
