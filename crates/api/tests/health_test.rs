//! API Health Tests

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Router;

use api::test_helpers::{call_router, create_test_app_state, create_test_router};

#[tokio::test]
async fn test_health_check() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert!(json["version"].as_str().is_some());
}

#[tokio::test]
async fn test_version_endpoint() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/version")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "Agent Playground API");
    assert!(json["version"].as_str().is_some());
}

#[tokio::test]
async fn test_api_docs_endpoint() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/docs")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Agent Playground API"));
    assert!(html.contains("System Endpoints"));
    assert!(html.contains("Brain API"));
    assert!(html.contains("Engine API"));
}

#[tokio::test]
async fn test_dashboard_stats() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/dashboard/stats")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["dashboardStats"].is_array());
    assert!(json["activeSimulations"].is_array());
}
