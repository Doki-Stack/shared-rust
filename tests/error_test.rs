use doki_shared::{Error, ErrorCode};

#[test]
fn test_not_found() {
    let e = Error::not_found("task-123");
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::NotFound);
    assert!(!retryable);
}

#[test]
fn test_bad_request() {
    let e = Error::bad_request("invalid input");
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::BadRequest);
    assert!(!retryable);
}

#[test]
fn test_unauthorized() {
    let e = Error::unauthorized("no token");
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::Unauthorized);
    assert!(!retryable);
}

#[test]
fn test_forbidden() {
    let e = Error::forbidden("no access");
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::Forbidden);
    assert!(!retryable);
}

#[test]
fn test_internal() {
    let e = Error::internal("something broke");
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::InternalError);
    assert!(retryable);
}

#[test]
fn test_rate_limited() {
    let e = Error::rate_limited("too many requests");
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::RateLimited);
    assert!(retryable);
}

#[test]
fn test_status_codes() {
    assert_eq!(Error::not_found("x").status_code().as_u16(), 404);
    assert_eq!(Error::bad_request("x").status_code().as_u16(), 400);
    assert_eq!(Error::unauthorized("x").status_code().as_u16(), 401);
    assert_eq!(Error::forbidden("x").status_code().as_u16(), 403);
    assert_eq!(Error::internal("x").status_code().as_u16(), 500);
    assert_eq!(Error::rate_limited("x").status_code().as_u16(), 429);
}

#[test]
fn test_domain_constructor() {
    let e = Error::domain(ErrorCode::PolicyUnavailable, "policy down", true);
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::PolicyUnavailable);
    assert!(retryable);
    assert_eq!(e.status_code().as_u16(), 500);
}

#[test]
fn test_from_anyhow() {
    let anyhow_err = anyhow::anyhow!("something went wrong");
    let e: Error = anyhow_err.into();
    let (code, retryable) = e.code_and_retryable();
    assert_eq!(code, ErrorCode::InternalError);
    assert!(retryable);
}

#[test]
fn test_error_display() {
    let e = Error::not_found("task-abc");
    assert_eq!(e.to_string(), "Resource not found: task-abc");
}
