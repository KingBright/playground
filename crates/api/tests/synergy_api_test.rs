//! Synergy API Integration Tests

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Router;

use api::test_helpers::{call_router, create_test_app_state, create_test_router};

#[tokio::test]
async fn test_list_agents() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/synergy/agents")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["agents"].is_array());
    assert!(json["agents"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_register_agent() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request_body = serde_json::json!({
        "name": "Test Agent",
        "type": "universal",
        "description": "A test agent for integration testing",
        "capabilities": ["test", "mock"]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/synergy/agents")
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
    assert_eq!(json["name"], "Test Agent");
    assert_eq!(json["type"], "universal");
    assert_eq!(json["status"], "active");
    assert!(json["capabilities"].is_array());
}

#[tokio::test]
async fn test_unregister_agent() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .method("DELETE")
        .uri("/api/synergy/agents/test-agent-123")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["id"], "test-agent-123");
    assert_eq!(json["status"], "unregistered");
}

#[tokio::test]
async fn test_list_tasks() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/synergy/tasks")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["tasks"].is_array());
}

#[tokio::test]
async fn test_create_task() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request_body = serde_json::json!({
        "name": "Test Scheduled Task",
        "type": "scheduled",
        "schedule": "0 0 * * *"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/synergy/tasks")
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
    assert_eq!(json["name"], "Test Scheduled Task");
    assert_eq!(json["type"], "scheduled");
    assert_eq!(json["schedule"], "0 0 * * *");
    assert_eq!(json["status"], "active");
}

#[tokio::test]
async fn test_delete_task() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .method("DELETE")
        .uri("/api/synergy/tasks/test-task-123")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["id"], "test-task-123");
    assert_eq!(json["status"], "deleted");
}

#[tokio::test]
async fn test_create_manual_task() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request_body = serde_json::json!({
        "name": "Test Manual Task",
        "type": "manual"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/synergy/tasks")
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
    assert_eq!(json["name"], "Test Manual Task");
    assert_eq!(json["type"], "manual");
    assert!(json["schedule"].is_null());
}
