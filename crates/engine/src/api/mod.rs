//! Engine API - REST API for the Engine system
//!
//! This module provides HTTP endpoints for:
//! - Session management (create, start, pause, resume, stop)
//! - Agent management within sessions
//! - Workflow execution
//! - Environment state access
//! - Real-time updates via WebSocket

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::session::{Session, SessionConfig, SessionManager, SessionSnapshot, SessionStatus};
use crate::workflow::{WorkflowEngine, WorkflowScript};

/// Engine API state
#[derive(Clone, Debug)]
pub struct EngineApiState {
    /// Session manager
    pub session_manager: Arc<SessionManager>,
    /// Active workflow engines
    pub workflow_engines: Arc<RwLock<HashMap<Uuid, Arc<WorkflowEngine>>>>,
}

impl EngineApiState {
    /// Create new API state
    pub fn new() -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new()),
            workflow_engines: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for EngineApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Engine API
#[derive(Debug)]
pub struct EngineApi {
    state: EngineApiState,
}

impl EngineApi {
    /// Create new Engine API
    pub fn new() -> Self {
        Self {
            state: EngineApiState::new(),
        }
    }

    /// Create API router
    pub fn router(&self) -> Router {
        Router::new()
            // Health check
            .route("/health", get(health_check))
            // Session management
            .route("/sessions", post(create_session).get(list_sessions))
            .route("/sessions/:id", get(get_session).delete(delete_session))
            .route("/sessions/:id/start", post(start_session))
            .route("/sessions/:id/pause", post(pause_session))
            .route("/sessions/:id/resume", post(resume_session))
            .route("/sessions/:id/stop", post(complete_session))
            .route(
                "/sessions/:id/snapshots",
                get(list_snapshots).post(create_snapshot),
            )
            .route(
                "/sessions/:id/snapshots/:snapshot_id/restore",
                post(restore_snapshot),
            )
            // Environment state
            .route("/sessions/:id/state", get(get_session_state))
            // WebSocket for real-time updates
            .route("/sessions/:id/ws", get(session_websocket))
            // Workflow
            .route("/sessions/:id/workflows", post(execute_workflow))
            .with_state(self.state.clone())
    }
}

// =============================================================================
// Request/Response Types
// =============================================================================

/// Create session request
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// Session name
    pub name: String,
    /// Session description
    pub description: Option<String>,
    /// Environment type
    pub environment_type: String,
    /// Environment configuration
    #[serde(default)]
    pub environment_config: serde_json::Value,
}

/// Session response
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Session status
    pub status: String,
    /// Created at
    pub created_at: String,
}

/// Session list response
#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    /// List of sessions
    pub sessions: Vec<SessionResponse>,
    /// Total count
    pub total: usize,
}

/// Workflow execution request
#[derive(Debug, Deserialize)]
pub struct ExecuteWorkflowRequest {
    /// Workflow script
    pub script: String,
    /// Script language (default: rhai)
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "rhai".to_string()
}

/// Workflow execution response
#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    /// Execution ID
    pub execution_id: String,
    /// Status
    pub status: String,
    /// Result
    pub result: Option<serde_json::Value>,
}

/// Snapshot response
#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    /// Snapshot ID
    pub id: String,
    /// Created at
    pub created_at: String,
    /// Description
    pub description: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

/// Health response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Status
    pub status: String,
    /// Version
    pub version: String,
}

// =============================================================================
// Handlers
// =============================================================================

/// Health check
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: crate::ENGINE_VERSION.to_string(),
    })
}

/// Create new session
async fn create_session(
    State(state): State<EngineApiState>,
    Json(request): Json<CreateSessionRequest>,
) -> std::result::Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Clone name before moving into config
    let name = request.name.clone();

    // For now, we need to create a basic environment
    // In a full implementation, this would use environment_type to create appropriate environment
    let config = SessionConfig {
        name: request.name,
        description: request.description,
        ..Default::default()
    };

    // Create a basic environment using EnvironmentBuilder
    let env_builder =
        crate::environment::EnvironmentBuilder::new(&request.environment_type, "1.0.0");
    let environment = match env_builder.build() {
        Ok(env) => Box::new(env) as Box<dyn crate::environment::Environment>,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to create environment: {}", e),
                code: "ENVIRONMENT_ERROR".to_string(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    match state
        .session_manager
        .create_session(config, environment)
        .await
    {
        Ok(session) => {
            let stats = session.stats().await;
            Ok(Json(SessionResponse {
                id: session.id.to_string(),
                name: name.clone(),
                status: format!("{:?}", stats.status),
                created_at: stats.created_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to create session: {}", e),
                code: "SESSION_CREATE_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// List all sessions
async fn list_sessions(State(state): State<EngineApiState>) -> Json<SessionListResponse> {
    let session_ids = state.session_manager.list_sessions().await;
    let mut session_responses = Vec::new();

    for id in session_ids {
        if let Some(session) = state.session_manager.get_session(id).await {
            let stats = session.stats().await;
            session_responses.push(SessionResponse {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status),
                created_at: stats.created_at.to_rfc3339(),
            });
        }
    }

    let total = session_responses.len();

    Json(SessionListResponse {
        sessions: session_responses,
        total,
    })
}

/// Get session by ID
async fn get_session(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(session) => {
            let stats = session.stats().await;
            Ok(Json(SessionResponse {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status),
                created_at: stats.created_at.to_rfc3339(),
            }))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Delete session
async fn delete_session(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.delete_session(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to delete session: {}", e),
                code: "DELETE_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Start session
async fn start_session(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(session) => {
            if let Err(e) = session.start().await {
                let error_response = ErrorResponse {
                    error: format!("Failed to start session: {}", e),
                    code: "START_ERROR".to_string(),
                };
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
            }

            let stats = session.stats().await;
            Ok(Json(SessionResponse {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status),
                created_at: stats.created_at.to_rfc3339(),
            }))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Pause session
async fn pause_session(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(session) => {
            if let Err(e) = session.pause().await {
                let error_response = ErrorResponse {
                    error: format!("Failed to pause session: {}", e),
                    code: "PAUSE_ERROR".to_string(),
                };
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
            }

            let stats = session.stats().await;
            Ok(Json(SessionResponse {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status),
                created_at: stats.created_at.to_rfc3339(),
            }))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Resume session
async fn resume_session(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(session) => {
            if let Err(e) = session.resume().await {
                let error_response = ErrorResponse {
                    error: format!("Failed to resume session: {}", e),
                    code: "RESUME_ERROR".to_string(),
                };
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
            }

            let stats = session.stats().await;
            Ok(Json(SessionResponse {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status),
                created_at: stats.created_at.to_rfc3339(),
            }))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Complete session
async fn complete_session(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(session) => {
            if let Err(e) = session.complete().await {
                let error_response = ErrorResponse {
                    error: format!("Failed to complete session: {}", e),
                    code: "COMPLETE_ERROR".to_string(),
                };
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
            }

            let stats = session.stats().await;
            Ok(Json(SessionResponse {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status),
                created_at: stats.created_at.to_rfc3339(),
            }))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Get session state
async fn get_session_state(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(_session) => {
            // Placeholder - would get environment state in full implementation
            Ok(Json(serde_json::json!({
                "state": "placeholder",
                "session_id": id.to_string(),
            })))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// List snapshots for a session
async fn list_snapshots(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<Vec<SnapshotResponse>>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(session) => {
            let snapshots = session.list_snapshots().await;
            let responses: Vec<SnapshotResponse> = snapshots
                .into_iter()
                .map(|s| SnapshotResponse {
                    id: s.id.to_string(),
                    created_at: s.created_at.to_rfc3339(),
                    description: s.description,
                })
                .collect();
            Ok(Json(responses))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Create snapshot
async fn create_snapshot(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<SnapshotResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(session) => match session.create_snapshot("API snapshot").await {
            Ok(snapshot) => Ok(Json(SnapshotResponse {
                id: snapshot.id.to_string(),
                created_at: snapshot.created_at.to_rfc3339(),
                description: snapshot.description,
            })),
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("Failed to create snapshot: {}", e),
                    code: "SNAPSHOT_ERROR".to_string(),
                };
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
            }
        },
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Restore snapshot
async fn restore_snapshot(
    State(state): State<EngineApiState>,
    Path((id, snapshot_id)): Path<(Uuid, Uuid)>,
) -> std::result::Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(session) => {
            if let Err(e) = session.restore_snapshot(snapshot_id).await {
                let error_response = ErrorResponse {
                    error: format!("Failed to restore snapshot: {}", e),
                    code: "RESTORE_ERROR".to_string(),
                };
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
            }

            let stats = session.stats().await;
            Ok(Json(SessionResponse {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status),
                created_at: stats.created_at.to_rfc3339(),
            }))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// WebSocket handler for real-time session updates
async fn session_websocket(Path(id): Path<Uuid>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, id))
}

/// Handle WebSocket connection
async fn handle_websocket(mut socket: axum::extract::ws::WebSocket, session_id: Uuid) {
    use axum::extract::ws::Message;

    // Send initial connection message
    let _ = socket
        .send(Message::Text(format!(
            r#"{{"type": "connected", "session_id": "{}"}}"#,
            session_id
        )))
        .await;

    // In a full implementation, this would:
    // 1. Subscribe to session state changes
    // 2. Broadcast updates to all connected clients
    // 3. Handle client commands

    // For now, just echo back any messages
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            let response = format!(
                r#"{{"type": "echo", "session_id": "{}", "data": {}}}"#,
                session_id, text
            );
            let _ = socket.send(Message::Text(response)).await;
        }
    }
}

/// Execute workflow in session
async fn execute_workflow(
    State(state): State<EngineApiState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ExecuteWorkflowRequest>,
) -> std::result::Result<Json<WorkflowResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.session_manager.get_session(id).await {
        Some(_session) => {
            // Create workflow script
            let script = WorkflowScript {
                id: Uuid::new_v4(),
                name: "api_workflow".to_string(),
                language: request.language,
                script: request.script,
                description: None,
            };

            // For now, return a placeholder response
            // In full implementation, this would execute the workflow
            let _ = script;

            Ok(Json(WorkflowResponse {
                execution_id: Uuid::new_v4().to_string(),
                status: "completed".to_string(),
                result: Some(serde_json::json!({"message": "Workflow executed"})),
            }))
        }
        None => {
            let error_response = ErrorResponse {
                error: "Session not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_api_creation() {
        let api = EngineApi::new();
        let _router = api.router();
        // Router created successfully
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.status, "healthy");
    }
}
