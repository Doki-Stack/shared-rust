use doki_shared::models::*;
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
    assert_eq!(decoded.message, envelope.message);
    assert_eq!(decoded.retryable, envelope.retryable);
}

#[test]
fn test_error_envelope_skips_none() {
    let envelope = ErrorEnvelope {
        error_code: "BAD_REQUEST".to_string(),
        message: "bad".to_string(),
        trace_id: None,
        org_id: None,
        retryable: false,
    };
    let json = serde_json::to_string(&envelope).unwrap();
    assert!(!json.contains("trace_id"));
    assert!(!json.contains("org_id"));
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
    assert_eq!(json["branch"], "main");
    assert!(json["commit_sha"].is_null());
}

#[test]
fn test_scan_result_roundtrip() {
    let result = ScanResult {
        scan_id: Uuid::new_v4(),
        status: ScanStatus::Completed,
        context_files: vec![ContextFile {
            path: "main.tf".to_string(),
            content_type: "terraform".to_string(),
            size_bytes: 1024,
        }],
        terraform_files: vec!["main.tf".to_string()],
    };
    let json = serde_json::to_string(&result).unwrap();
    let decoded: ScanResult = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.status, ScanStatus::Completed);
    assert_eq!(decoded.context_files.len(), 1);
}

#[test]
fn test_plan_output_roundtrip() {
    let output = PlanOutput {
        plan_id: Uuid::new_v4(),
        status: ExecutionStatus::Completed,
        resource_changes: vec![ResourceChange {
            address: "aws_s3_bucket.main".to_string(),
            action: "create".to_string(),
            change_type: Some("managed".to_string()),
        }],
        raw_output: None,
    };
    let json = serde_json::to_string(&output).unwrap();
    let decoded: PlanOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.resource_changes.len(), 1);
    assert_eq!(decoded.resource_changes[0].action, "create");
}

#[test]
fn test_pagination_default() {
    let p = Pagination::default();
    assert_eq!(p.page, 1);
    assert_eq!(p.per_page, 20);
}

#[test]
fn test_api_response_format() {
    let resp = ApiResponse {
        data: vec!["item1", "item2"],
        pagination: Some(PaginationMeta {
            page: 1,
            per_page: 20,
            total: 2,
        }),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
    assert_eq!(json["pagination"]["total"], 2);
}

#[test]
fn test_execution_status_serde() {
    let status = ExecutionStatus::InProgress;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"in_progress\"");
    let decoded: ExecutionStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, ExecutionStatus::InProgress);
}

#[test]
fn test_audit_action_serde() {
    let action = AuditAction::Create;
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"create\"");
}
