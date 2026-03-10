use axum::{body::Body, http::Request, http::StatusCode, routing::get, Router};
use tower::ServiceExt;
use uuid::Uuid;

use doki_shared::extractors::OrgId;

fn app() -> Router {
    Router::new().route(
        "/",
        get(|OrgId(org_id): OrgId| async move { org_id.to_string() }),
    )
}

#[tokio::test]
async fn test_valid_org_id() {
    let uuid = Uuid::new_v4();
    let req = Request::builder()
        .uri("/")
        .header("X-Org-Id", uuid.to_string())
        .body(Body::empty())
        .unwrap();

    let response = app().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_header() {
    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = app().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_uuid() {
    let req = Request::builder()
        .uri("/")
        .header("X-Org-Id", "not-a-uuid")
        .body(Body::empty())
        .unwrap();

    let response = app().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_empty_header() {
    let req = Request::builder()
        .uri("/")
        .header("X-Org-Id", "")
        .body(Body::empty())
        .unwrap();

    let response = app().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_whitespace_header() {
    let req = Request::builder()
        .uri("/")
        .header("X-Org-Id", "   ")
        .body(Body::empty())
        .unwrap();

    let response = app().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
