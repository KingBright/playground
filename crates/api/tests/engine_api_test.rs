//! Engine API Integration Tests

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Router;

use api::test_helpers::{call_router, create_test_app_state, create_test_router};

#[tokio::test]
async fn test_list_sessions() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/engine/sessions")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["sessions"].is_array());
}

#[tokio::test]
async fn test_list_environments() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request = Request::builder()
        .uri("/api/engine/environments")
        .body(Body::empty())
        .unwrap();

    let response: Response<Body> = call_router(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["environments"].is_array());
    assert!(json["environments"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_create_session() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    let request_body = serde_json::json!({
        "name": "Test Session",
        "environment_type": "chess"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/engine/sessions")
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
    assert_eq!(json["name"], "Test Session");
    assert!(json["status"].as_str().is_some());
    assert!(json["created_at"].as_str().is_some());
}

#[tokio::test]
async fn test_get_session() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    // First create a session
    let request_body = serde_json::json!({
        "name": "Test Session for Get",
        "environment_type": "chess"
    });

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/engine/sessions")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let create_response: Response<Body> = call_router(&mut app, create_request).await;

    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let session_id = create_json["id"].as_str().unwrap();

    // Now get the session
    let get_request = Request::builder()
        .uri(&format!("/api/engine/sessions/{}", session_id))
        .body(Body::empty())
        .unwrap();

    let get_response: Response<Body> = call_router(&mut app, get_request).await;

    assert_eq!(get_response.status(), StatusCode::OK);

    let get_body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let get_json: serde_json::Value = serde_json::from_slice(&get_body).unwrap();

    assert_eq!(get_json["id"], session_id);
    assert!(get_json["status"].as_str().is_some());
}

#[tokio::test]
async fn test_session_lifecycle() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    // Create a session
    let request_body = serde_json::json!({
        "name": "Lifecycle Test Session",
        "environment_type": "chess"
    });

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/engine/sessions")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let create_response: Response<Body> = call_router(&mut app, create_request).await;

    assert_eq!(create_response.status(), StatusCode::OK);

    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let session_id = create_json["id"].as_str().unwrap();

    // Start the session
    let start_request = Request::builder()
        .method("POST")
        .uri(&format!("/api/engine/sessions/{}/start", session_id))
        .body(Body::empty())
        .unwrap();

    let start_response: Response<Body> = call_router(&mut app, start_request).await;

    assert_eq!(start_response.status(), StatusCode::OK);

    let start_body = axum::body::to_bytes(start_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).unwrap();
    assert_eq!(start_json["status"], "running");

    // Pause the session
    let pause_request = Request::builder()
        .method("POST")
        .uri(&format!("/api/engine/sessions/{}/pause", session_id))
        .body(Body::empty())
        .unwrap();

    let pause_response: Response<Body> = call_router(&mut app, pause_request).await;

    assert_eq!(pause_response.status(), StatusCode::OK);

    let pause_body = axum::body::to_bytes(pause_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let pause_json: serde_json::Value = serde_json::from_slice(&pause_body).unwrap();
    assert_eq!(pause_json["status"], "paused");

    // Stop the session
    let stop_request = Request::builder()
        .method("POST")
        .uri(&format!("/api/engine/sessions/{}/stop", session_id))
        .body(Body::empty())
        .unwrap();

    let stop_response: Response<Body> = call_router(&mut app, stop_request).await;

    assert_eq!(stop_response.status(), StatusCode::OK);

    let stop_body = axum::body::to_bytes(stop_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stop_json: serde_json::Value = serde_json::from_slice(&stop_body).unwrap();
    assert_eq!(stop_json["status"], "completed");

    // Delete the session
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/engine/sessions/{}", session_id))
        .body(Body::empty())
        .unwrap();

    let delete_response: Response<Body> = call_router(&mut app, delete_request).await;

    assert_eq!(delete_response.status(), StatusCode::OK);

    let delete_body = axum::body::to_bytes(delete_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let delete_json: serde_json::Value = serde_json::from_slice(&delete_body).unwrap();
    assert_eq!(delete_json["status"], "deleted");
}
