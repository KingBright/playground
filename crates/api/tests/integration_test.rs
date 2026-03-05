//! Full Lifecycle Integration Tests

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Router;

use api::test_helpers::{call_router, create_test_app_state, create_test_router};

#[tokio::test]
async fn test_full_platform_lifecycle() {
    let state = create_test_app_state().await;
    let mut app: Router = create_test_router(state);

    // 1. Check system health
    let req_health = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();
    let res_health: Response<Body> = call_router(&mut app, req_health).await;
    assert_eq!(res_health.status(), StatusCode::OK);

    // 2. Register a new agent
    let req_agent_body = serde_json::json!({
        "name": "Integration Test Agent",
        "type": "universal",
        "description": "An agent for full lifecycle integration test",
        "capabilities": ["test", "analyze"]
    });
    let req_agent = Request::builder()
        .method("POST")
        .uri("/api/synergy/agents")
        .header("content-type", "application/json")
        .body(Body::from(req_agent_body.to_string()))
        .unwrap();
    let res_agent: Response<Body> = call_router(&mut app, req_agent).await;
    assert_eq!(res_agent.status(), StatusCode::OK);

    // 3. Create a new session
    let req_session_body = serde_json::json!({
        "name": "Integration Test Session",
        "environment_type": "chess"
    });
    let req_session = Request::builder()
        .method("POST")
        .uri("/api/engine/sessions")
        .header("content-type", "application/json")
        .body(Body::from(req_session_body.to_string()))
        .unwrap();
    let res_session: Response<Body> = call_router(&mut app, req_session).await;
    assert_eq!(res_session.status(), StatusCode::OK);
    let session_body = axum::body::to_bytes(res_session.into_body(), usize::MAX)
        .await
        .unwrap();
    let session_json: serde_json::Value = serde_json::from_slice(&session_body).unwrap();
    let session_id = session_json["id"].as_str().unwrap();

    // 4. Start the session
    let req_start = Request::builder()
        .method("POST")
        .uri(&format!("/api/engine/sessions/{}/start", session_id))
        .body(Body::empty())
        .unwrap();
    let res_start: Response<Body> = call_router(&mut app, req_start).await;
    assert_eq!(res_start.status(), StatusCode::OK);

    // 5. Create a new task
    let req_task_body = serde_json::json!({
        "name": "Analyze session results",
        "type": "manual"
    });
    let req_task = Request::builder()
        .method("POST")
        .uri("/api/synergy/tasks")
        .header("content-type", "application/json")
        .body(Body::from(req_task_body.to_string()))
        .unwrap();
    let res_task: Response<Body> = call_router(&mut app, req_task).await;
    assert_eq!(res_task.status(), StatusCode::OK);

    // 6. Add knowledge slice
    let req_knowledge_body = serde_json::json!({
        "name": "Session Results Log",
        "tags": ["integration", "results"]
    });
    let req_knowledge = Request::builder()
        .method("POST")
        .uri("/api/brain/knowledge")
        .header("content-type", "application/json")
        .body(Body::from(req_knowledge_body.to_string()))
        .unwrap();
    let res_knowledge: Response<Body> = call_router(&mut app, req_knowledge).await;
    assert_eq!(res_knowledge.status(), StatusCode::OK);
    let knowledge_body = axum::body::to_bytes(res_knowledge.into_body(), usize::MAX)
        .await
        .unwrap();
    let knowledge_json: serde_json::Value = serde_json::from_slice(&knowledge_body).unwrap();
    let knowledge_id = knowledge_json["id"].as_str().unwrap();

    // 7. Stop the session
    let req_stop = Request::builder()
        .method("POST")
        .uri(&format!("/api/engine/sessions/{}/stop", session_id))
        .body(Body::empty())
        .unwrap();
    let res_stop: Response<Body> = call_router(&mut app, req_stop).await;
    assert_eq!(res_stop.status(), StatusCode::OK);

    // 8. Delete knowledge slice
    let req_del_knowledge = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/brain/knowledge/{}", knowledge_id))
        .body(Body::empty())
        .unwrap();
    let res_del_knowledge: Response<Body> = call_router(&mut app, req_del_knowledge).await;
    assert_eq!(res_del_knowledge.status(), StatusCode::OK);

    // 9. Delete the session
    let req_del_session = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/engine/sessions/{}", session_id))
        .body(Body::empty())
        .unwrap();
    let res_del_session: Response<Body> = call_router(&mut app, req_del_session).await;
    assert_eq!(res_del_session.status(), StatusCode::OK);
}
