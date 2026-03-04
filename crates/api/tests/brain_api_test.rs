//! Brain API Integration Tests

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Router;

use api::test_helpers::{call_router, create_test_app_state, create_test_router};

#[tokio::test]
async fn test_list_knowledge_slices() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/brain/knowledge")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["slices"].is_array());
}

#[tokio::test]
async fn test_create_knowledge_slice() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request_body = serde_json::json!({
        "name": "Test Knowledge Slice",
        "tags": ["test", "sample"]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/brain/knowledge")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["id"].as_str().is_some());
    assert_eq!(json["name"], "Test Knowledge Slice");
    assert_eq!(json["status"], "created");
}

#[tokio::test]
async fn test_delete_knowledge_slice() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .method("DELETE")
        .uri("/api/brain/knowledge/test-id-123")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["id"], "test-id-123");
    assert_eq!(json["status"], "deleted");
}

#[tokio::test]
async fn test_brain_health() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/brain/health")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["status"].as_str().is_some());
    assert!(json["hot_memory"].is_boolean());
    assert!(json["vector_memory"].is_boolean());
    assert!(json["graph_memory"].is_boolean());
    assert!(json["raw_archive"].is_boolean());
}
