//! Agent Playground API Server
//!
//! Unified API gateway for the Agent Playground platform.

mod models;

use anyhow::Result;
use axum::{
    body::Body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Path, State},
    http::{header, Method, Response, StatusCode},
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Json, Router,
};
use clap::Parser;
use models::*;
use rust_embed::RustEmbed;
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::System;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info, warn};

// Import module components
use brain::storage::{
    graph_memory::InMemoryGraphStore,
    hot_memory::InMemoryHotMemory,
    raw_archive::{FileSystemRawArchive, RawArchiveConfig},
    vector_memory::InMemoryVectorStore,
    GraphMemoryBackend, HotMemoryBackend, RawArchiveBackend, UnifiedMemory, VectorMemoryBackend,
};
use engine::session::SessionManager;

/// Embedded static files (for release builds)
#[derive(RustEmbed)]
#[folder = "static/"]
#[prefix = "/"]
struct EmbeddedAssets;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: String,
    pub static_dir: String,
    pub api_only: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:8080".to_string(),
            static_dir: "crates/api/static".to_string(),
            api_only: false,
        }
    }
}

use synergy::registry::AgentRegistry;
/// Application state with real module integrations
use synergy::scheduler::MissionControl;

#[derive(Clone)]
pub struct AppState {
    pub config: ServerConfig,
    pub session_manager: Arc<SessionManager>,
    pub brain_memory: Arc<UnifiedMemory>,
    pub registry: Arc<AgentRegistry>,
    pub mission_control: Arc<MissionControl>,
}

/// Agent Playground API Server
#[derive(Parser, Debug)]
#[command(name = "agent-playground")]
#[command(about = "Agent Playground - AI Simulation Platform")]
struct Cli {
    /// Server bind address
    #[arg(short, long, default_value = "0.0.0.0:8080", env = "API_BIND")]
    bind: String,

    /// Log level
    #[arg(short, long, default_value = "info", env = "RUST_LOG")]
    log_level: String,

    /// Static files directory
    #[arg(short, long, default_value = "crates/api/static", env = "STATIC_DIR")]
    static_dir: String,

    /// API only mode (disable static files)
    #[arg(long, env = "API_ONLY")]
    api_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&cli.log_level)
        .with_target(false)
        .init();

    info!("🚀 Agent Playground API Server starting...");
    info!("📡 Binding to: {}", cli.bind);

    // Initialize modules
    let app_state = init_app_state(&cli).await?;

    // API routes
    let api_routes = create_api_routes();

    // Static file routes
    let static_routes = if app_state.config.api_only {
        info!("API-only mode: static file serving disabled");
        Router::new()
    } else {
        create_static_routes()
    };

    // Build app
    let app = Router::new()
        .nest("/api", api_routes)
        .merge(static_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Parse bind address
    let addr: SocketAddr = cli.bind.parse()?;

    info!("✅ Server ready!");

    // 检查静态文件状态
    let static_dir = PathBuf::from(&cli.static_dir);
    let has_web_ui =
        static_dir.exists() && static_dir.is_dir() && static_dir.join("index.html").exists();

    if has_web_ui {
        // 生产模式：静态文件已构建
        info!("🌐 Web UI: http://{}", addr);
    } else {
        // 开发模式：前端在 Vite dev server
        info!("🌐 Web UI: http://localhost:5173 (Vite Dev Server)");
        info!("💡 Run './manage.sh build' to serve Web UI from API server");
    }
    info!("📊 API Docs: http://{}/api/docs", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize application state with real module connections
async fn init_app_state(cli: &Cli) -> Result<AppState> {
    info!("Initializing modules...");

    // Initialize Engine
    let session_manager = Arc::new(SessionManager::new());
    info!("✓ Engine module initialized");

    // Initialize Brain storage
    let hot = Arc::new(InMemoryHotMemory::new()) as Arc<dyn HotMemoryBackend>;
    let vector =
        Arc::new(InMemoryVectorStore::with_default_config()) as Arc<dyn VectorMemoryBackend>;
    let graph = Arc::new(InMemoryGraphStore::new()) as Arc<dyn GraphMemoryBackend>;

    // Initialize raw archive with temp directory
    let raw_dir = std::env::temp_dir().join("agent-playground-raw-archive");
    let raw = Arc::new(
        FileSystemRawArchive::new(RawArchiveConfig {
            storage_dir: raw_dir,
            ..Default::default()
        })
        .await?,
    ) as Arc<dyn RawArchiveBackend>;

    let brain_memory = Arc::new(UnifiedMemory::new(hot, vector, graph, raw));
    info!("✓ Brain module initialized");

    // Create config
    let config = ServerConfig {
        bind: cli.bind.clone(),
        static_dir: cli.static_dir.clone(),
        api_only: cli.api_only,
    };

    let registry = Arc::new(AgentRegistry::new());
    let mission_control = Arc::new(MissionControl::new(
        registry.clone(),
        synergy::scheduler::SchedulerConfig::default(),
    ));

    // Start the scheduler
    let mc_clone = mission_control.clone();
    tokio::spawn(async move {
        mc_clone.start_scheduler().await;
    });

    info!("✓ Synergy module initialized and scheduler started");

    Ok(AppState {
        config,
        session_manager,
        brain_memory,
        registry,
        mission_control,
    })
}

/// Create API routes
fn create_api_routes() -> Router<AppState> {
    Router::new()
        // System endpoints
        .route("/health", get(health_check))
        .route("/version", get(version))
        .route("/docs", get(api_docs))
        // Dashboard API
        .route("/dashboard/stats", get(get_dashboard_stats))
        // Brain API
        .route(
            "/brain/knowledge",
            get(list_knowledge).post(create_knowledge),
        )
        .route("/brain/knowledge/:id", delete(delete_knowledge))
        .route("/brain/health", get(brain_health))
        // Engine API
        .route("/engine/sessions", get(list_sessions).post(create_session))
        .route(
            "/engine/sessions/:id",
            get(get_session).delete(delete_session),
        )
        .route("/engine/sessions/:id/start", post(start_session))
        .route("/engine/sessions/:id/pause", post(pause_session))
        .route("/engine/sessions/:id/stop", post(stop_session))
        .route("/engine/environments", get(list_environments))
        // Synergy API
        .route("/synergy/agents", get(list_agents).post(register_agent))
        .route("/synergy/agents/:id", delete(unregister_agent))
        .route("/synergy/tasks", get(list_tasks).post(create_task))
        .route("/synergy/tasks/:id", delete(delete_task))
        // WebSocket endpoints
        .route("/ws/sessions/:id", get(session_websocket))
        .route("/ws/missions", get(missions_websocket))
        .route("/ws/system", get(system_websocket))
}

/// Create static file routes
fn create_static_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(serve_index))
        .fallback(serve_static_or_embedded)
}

// API Handlers

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn version() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "Agent Playground API",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn api_docs() -> Html<String> {
    Html(get_api_docs_html())
}

async fn get_dashboard_stats(State(state): State<AppState>) -> impl IntoResponse {
    // Get real session data from Engine
    let session_ids = state.session_manager.list_sessions().await;
    let mut active_simulations = Vec::new();

    for id in session_ids {
        if let Some(session) = state.session_manager.get_session(id).await {
            let stats = session.stats().await;
            active_simulations.push(Simulation {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status).to_lowercase(),
                environment: "Default".to_string(),
                agents: vec![],
                start_time: stats.created_at.to_rfc3339(),
                progress: 0,
            });
        }
    }

    // Build stats based on real data
    let stats = vec![
        StatCardData {
            label: "Active Sessions".to_string(),
            value: active_simulations.len().to_string(),
            change: None,
            change_label: None,
            icon: "smart_toy".to_string(),
            icon_color: "bg-blue-500/10 text-primary".to_string(),
            trend: None,
        },
        StatCardData {
            label: "System Status".to_string(),
            value: "Online".to_string(),
            change: None,
            change_label: Some("All systems operational".to_string()),
            icon: "check_circle".to_string(),
            icon_color: "bg-green-500/10 text-green-400".to_string(),
            trend: Some("stable".to_string()),
        },
        StatCardData {
            label: "Memory Backend".to_string(),
            value: "In-Memory".to_string(),
            change: None,
            change_label: None,
            icon: "memory".to_string(),
            icon_color: "bg-purple-500/10 text-purple-400".to_string(),
            trend: None,
        },
        StatCardData {
            label: "API Version".to_string(),
            value: env!("CARGO_PKG_VERSION").to_string(),
            change: None,
            change_label: None,
            icon: "api".to_string(),
            icon_color: "bg-orange-500/10 text-orange-400".to_string(),
            trend: None,
        },
    ];

    Json(SystemStats {
        dashboard_stats: stats,
        active_simulations,
    })
}

async fn brain_health(State(state): State<AppState>) -> impl IntoResponse {
    match state.brain_memory.health_check().await {
        Ok(health) => Json(serde_json::json!({
            "status": if health.overall { "healthy" } else { "unhealthy" },
            "hot_memory": health.hot_memory,
            "vector_memory": health.vector_memory,
            "graph_memory": health.graph_memory,
            "raw_archive": health.raw_archive,
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": format!("{}", e),
        })),
    }
}

async fn list_knowledge(State(state): State<AppState>) -> impl IntoResponse {
    // Basic query to check real memory state
    let mut slices = Vec::new();

    // Attempt to get stats from brain (currently minimal, but we can return actual size)
    let node_count = 0; // TODO: Fetch real count from Graph DB

    slices.push(KnowledgeSlice {
        id: "system-memory-default".to_string(),
        name: "Primary Brain Memory".to_string(),
        node_count,
        status: "active".to_string(),
        last_updated: chrono::Utc::now().to_rfc3339(),
        tags: vec!["core".to_string(), "graph".to_string()],
    });

    Json(KnowledgeListResponse { slices })
}

async fn list_sessions(State(state): State<AppState>) -> impl IntoResponse {
    let session_ids = state.session_manager.list_sessions().await;
    let mut sessions = Vec::new();

    for id in session_ids {
        if let Some(session) = state.session_manager.get_session(id).await {
            let stats = session.stats().await;
            sessions.push(Simulation {
                id: session.id.to_string(),
                name: format!("Session {}", session.id),
                status: format!("{:?}", stats.status).to_lowercase(),
                environment: "Default".to_string(),
                agents: vec![],
                start_time: stats.created_at.to_rfc3339(),
                progress: 0,
            });
        }
    }

    Json(serde_json::json!({
        "sessions": sessions
    }))
}

async fn list_environments() -> impl IntoResponse {
    Json(serde_json::json!({
        "environments": [
            {"id": "chess", "name": "Chess Game"},
            {"id": "debate", "name": "Debate Hall"},
            {"id": "news-studio", "name": "News Studio"},
            {"id": "trading-floor", "name": "Trading Floor"}
        ]
    }))
}

async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    let active_agent_names = state.registry.list().await;
    let mut agents = Vec::new();

    for name in active_agent_names {
        if let Some(agent_def) = state.registry.get(&name).await {
            agents.push(Agent {
                id: name.clone(), // using name as id for simplicity since registry uses name as key
                name: agent_def.name.clone(),
                type_: format!("{:?}", agent_def.agent_type).to_lowercase(),
                description: agent_def.description.clone().unwrap_or_default(),
                capabilities: vec![], // Not stored in AgentDefinition directly
                status: "active".to_string(),
                version: "1.0.0".to_string(), // placeholder
                icon: Some("smart_toy".to_string()),
            });
        }
    }

    Json(AgentListResponse { agents })
}

async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let missions = state.mission_control.get_active_missions().await;
    let mut tasks = Vec::new();

    for m in missions {
        tasks.push(ScheduledTask {
            id: m.id.to_string(),
            name: m.name.clone(),
            type_: "scheduled".to_string(),
            status: "active".to_string(),
            schedule: Some("System Scheduled".to_string()),
            last_run: None,
            next_run: None,
        });
    }

    Json(TaskListResponse { tasks })
}

// ============================================================================
// Additional API Handlers (POST/PUT/DELETE)
// ============================================================================

use serde::Deserialize;
use uuid::Uuid;

/// Request to create a knowledge slice
#[derive(Debug, Deserialize)]
struct CreateKnowledgeRequest {
    name: String,
    tags: Vec<String>,
}

/// Request to create a session
#[derive(Debug, Deserialize)]
struct CreateSessionRequest {
    name: String,
    environment_type: String,
}

/// Request to register an agent
#[derive(Debug, Deserialize)]
struct RegisterAgentRequest {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    description: String,
    capabilities: Vec<String>,
}

/// Request to create a task
#[derive(Debug, Deserialize)]
struct CreateTaskRequest {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    schedule: Option<String>,
}

// Brain API handlers

async fn create_knowledge(Json(request): Json<CreateKnowledgeRequest>) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    Json(serde_json::json!({
        "id": id,
        "name": request.name,
        "tags": request.tags,
        "status": "created",
        "message": "Knowledge slice created successfully"
    }))
}

async fn delete_knowledge(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "id": id,
        "status": "deleted",
        "message": "Knowledge slice deleted successfully"
    }))
}

// Engine API handlers

async fn create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    use engine::environment::EnvironmentBuilder;
    use engine::session::SessionConfig;

    let config = SessionConfig {
        name: request.name.clone(),
        description: None,
        ..Default::default()
    };

    let env_builder = EnvironmentBuilder::new(&request.environment_type, "1.0.0");
    match env_builder.build() {
        Ok(environment) => {
            match state
                .session_manager
                .create_session(
                    config,
                    Box::new(environment) as Box<dyn engine::environment::Environment>,
                )
                .await
            {
                Ok(session) => {
                    let stats = session.stats().await;
                    Json(serde_json::json!({
                        "id": session.id.to_string(),
                        "name": request.name,
                        "status": format!("{:?}", stats.status).to_lowercase(),
                        "created_at": stats.created_at.to_rfc3339(),
                        "message": "Session created successfully"
                    }))
                    .into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to create session: {}", e),
                        "code": "SESSION_CREATE_ERROR"
                    })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Failed to create environment: {}", e),
                "code": "ENVIRONMENT_ERROR"
            })),
        )
            .into_response(),
    }
}

async fn get_session(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match id.parse::<Uuid>() {
        Ok(uuid) => {
            if let Some(session) = state.session_manager.get_session(uuid).await {
                let stats = session.stats().await;
                Json(serde_json::json!({
                    "id": session.id.to_string(),
                    "name": format!("Session {}", session.id),
                    "status": format!("{:?}", stats.status).to_lowercase(),
                    "created_at": stats.created_at.to_rfc3339(),
                    "environment": "Default"
                }))
                .into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Session not found",
                        "code": "NOT_FOUND"
                    })),
                )
                    .into_response()
            }
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid session ID",
                "code": "INVALID_ID"
            })),
        )
            .into_response(),
    }
}

async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match id.parse::<Uuid>() {
        Ok(uuid) => match state.session_manager.delete_session(uuid).await {
            Ok(true) => Json(serde_json::json!({
                "id": id,
                "status": "deleted",
                "message": "Session deleted successfully"
            }))
            .into_response(),
            Ok(false) => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Session not found",
                    "code": "NOT_FOUND"
                })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to delete session: {}", e),
                    "code": "DELETE_ERROR"
                })),
            )
                .into_response(),
        },
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid session ID",
                "code": "INVALID_ID"
            })),
        )
            .into_response(),
    }
}

async fn start_session(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match id.parse::<Uuid>() {
        Ok(uuid) => {
            if let Some(session) = state.session_manager.get_session(uuid).await {
                match session.start().await {
                    Ok(()) => {
                        let stats = session.stats().await;
                        Json(serde_json::json!({
                            "id": id,
                            "status": format!("{:?}", stats.status).to_lowercase(),
                            "message": "Session started successfully"
                        }))
                        .into_response()
                    }
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to start session: {}", e),
                            "code": "START_ERROR"
                        })),
                    )
                        .into_response(),
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Session not found",
                        "code": "NOT_FOUND"
                    })),
                )
                    .into_response()
            }
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid session ID",
                "code": "INVALID_ID"
            })),
        )
            .into_response(),
    }
}

async fn pause_session(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match id.parse::<Uuid>() {
        Ok(uuid) => {
            if let Some(session) = state.session_manager.get_session(uuid).await {
                match session.pause().await {
                    Ok(()) => {
                        let stats = session.stats().await;
                        Json(serde_json::json!({
                            "id": id,
                            "status": format!("{:?}", stats.status).to_lowercase(),
                            "message": "Session paused successfully"
                        }))
                        .into_response()
                    }
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to pause session: {}", e),
                            "code": "PAUSE_ERROR"
                        })),
                    )
                        .into_response(),
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Session not found",
                        "code": "NOT_FOUND"
                    })),
                )
                    .into_response()
            }
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid session ID",
                "code": "INVALID_ID"
            })),
        )
            .into_response(),
    }
}

async fn stop_session(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match id.parse::<Uuid>() {
        Ok(uuid) => {
            if let Some(session) = state.session_manager.get_session(uuid).await {
                match session.complete().await {
                    Ok(()) => {
                        let stats = session.stats().await;
                        Json(serde_json::json!({
                            "id": id,
                            "status": format!("{:?}", stats.status).to_lowercase(),
                            "message": "Session stopped successfully"
                        }))
                        .into_response()
                    }
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to stop session: {}", e),
                            "code": "STOP_ERROR"
                        })),
                    )
                        .into_response(),
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Session not found",
                        "code": "NOT_FOUND"
                    })),
                )
                    .into_response()
            }
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid session ID",
                "code": "INVALID_ID"
            })),
        )
            .into_response(),
    }
}

// Synergy API handlers

async fn register_agent(Json(request): Json<RegisterAgentRequest>) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    Json(serde_json::json!({
        "id": id,
        "name": request.name,
        "type": request.type_,
        "description": request.description,
        "capabilities": request.capabilities,
        "status": "active",
        "version": "1.0.0",
        "message": "Agent registered successfully"
    }))
}

async fn unregister_agent(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "id": id,
        "status": "unregistered",
        "message": "Agent unregistered successfully"
    }))
}

async fn create_task(Json(request): Json<CreateTaskRequest>) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    Json(serde_json::json!({
        "id": id,
        "name": request.name,
        "type": request.type_,
        "schedule": request.schedule,
        "status": "active",
        "message": "Task created successfully"
    }))
}

async fn delete_task(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "id": id,
        "status": "deleted",
        "message": "Task deleted successfully"
    }))
}

// Static file handlers

async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    let index_path = PathBuf::from(&state.config.static_dir).join("index.html");

    match tokio::fs::read_to_string(&index_path).await {
        Ok(content) => Html(content).into_response(),
        Err(_) => {
            // Try embedded assets
            match EmbeddedAssets::get("/index.html") {
                Some(file) => {
                    let content = String::from_utf8_lossy(file.data.as_ref());
                    Html(content.to_string()).into_response()
                }
                None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
            }
        }
    }
}

async fn serve_static_or_embedded(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let path = request.uri().path().trim_start_matches('/');

    // Try to serve from static directory first
    if !path.is_empty() {
        let file_path = PathBuf::from(&state.config.static_dir).join(path);

        if file_path.exists() {
            // Security check
            if let Ok(canonical_root) = tokio::fs::canonicalize(&state.config.static_dir).await {
                if let Ok(canonical_file) = tokio::fs::canonicalize(&file_path).await {
                    if canonical_file.starts_with(&canonical_root) {
                        if let Ok(content) = tokio::fs::read(&file_path).await {
                            let mime = mime_guess::from_path(path).first_or_octet_stream();
                            return Response::builder()
                                .header(header::CONTENT_TYPE, mime.as_ref())
                                .body(Body::from(content))
                                .unwrap();
                        }
                    }
                }
            }
        }

        // Try embedded assets
        let embedded_path = format!("/{}", path);
        if let Some(file) = EmbeddedAssets::get(&embedded_path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(file.data))
                .unwrap();
        }
    }

    // For SPA routing, return index.html for non-file paths
    if !path.contains('.') {
        let index_path = PathBuf::from(&state.config.static_dir).join("index.html");
        if let Ok(content) = tokio::fs::read_to_string(&index_path).await {
            return Html(content).into_response();
        }

        if let Some(file) = EmbeddedAssets::get("/index.html") {
            let content = String::from_utf8_lossy(file.data.as_ref());
            return Html(content.to_string()).into_response();
        }
    }

    (StatusCode::NOT_FOUND, "File not found").into_response()
}

// ============================================================================
// WebSocket Handlers
// ============================================================================

use tokio::time::{interval, Duration};

/// Session WebSocket - 实时推送Session状态
async fn session_websocket(
    Path(session_id): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_session_socket(socket, session_id, state))
}

async fn handle_session_socket(mut socket: WebSocket, session_id: String, state: AppState) {
    info!("WebSocket connected for session: {}", session_id);

    // 解析session ID
    let session_uuid = match uuid::Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => {
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"error": "Invalid session ID"}).to_string(),
                ))
                .await;
            return;
        }
    };

    // 发送欢迎消息
    let welcome = serde_json::json!({
        "type": "connected",
        "session_id": session_id,
        "message": "Session WebSocket connected"
    });
    if socket
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        return;
    }

    // 定期推送session状态
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // 获取session状态
                let status = if let Some(session) = state.session_manager.get_session(session_uuid).await {
                    let stats = session.stats().await;
                    serde_json::json!({
                        "type": "session_status",
                        "session_id": session_id,
                        "status": format!("{:?}", stats.status),
                        "agent_count": stats.agent_count,
                        "snapshot_count": stats.snapshot_count,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                } else {
                    serde_json::json!({
                        "type": "error",
                        "message": "Session not found"
                    })
                };

                if socket.send(Message::Text(status.to_string())).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // 处理客户端消息
                        info!("Received WebSocket message: {}", text);

                        // 回复确认
                        let ack = serde_json::json!({
                            "type": "ack",
                            "received": text
                        });
                        if socket.send(Message::Text(ack.to_string())).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("WebSocket disconnected for session: {}", session_id);
}

/// Missions WebSocket - 实时推送任务执行进度
async fn missions_websocket(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_missions_socket)
}

async fn handle_missions_socket(mut socket: WebSocket) {
    info!("Missions WebSocket connected");

    // 发送欢迎消息
    let welcome = serde_json::json!({
        "type": "connected",
        "message": "Missions WebSocket connected",
        "subscriptions": ["mission_updates", "execution_progress"]
    });
    if socket
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        return;
    }

    // 定期推送系统状态
    let mut ticker = interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let heartbeat = serde_json::json!({
                    "type": "heartbeat",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "active_missions": 0
                });

                if socket.send(Message::Text(heartbeat.to_string())).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // 处理客户端消息
                        if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                            let response = match cmd.get("action").and_then(|a| a.as_str()) {
                                Some("subscribe") => {
                                    serde_json::json!({
                                        "type": "subscribed",
                                        "channel": cmd.get("channel").unwrap_or(&serde_json::json!("all"))
                                    })
                                }
                                _ => {
                                    serde_json::json!({
                                        "type": "ack",
                                        "received": text
                                    })
                                }
                            };

                            if socket.send(Message::Text(response.to_string())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("Missions WebSocket disconnected");
}

fn get_api_docs_html() -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Agent Playground API Docs</title>
    <style>
        body {{ font-family: system-ui; max-width: 800px; margin: 40px auto; padding: 20px; }}
        h1 {{ color: #1152d4; }}
        code {{ background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }}
        .endpoint {{ margin: 20px 0; padding: 15px; background: #f9f9f9; border-radius: 8px; }}
        .method {{ color: #10b981; font-weight: bold; }}
        .section {{ margin-top: 30px; }}
    </style>
</head>
<body>
    <h1>🤖 Agent Playground API</h1>
    <p>Version: {}</p>

    <div class="section">
        <h2>System Endpoints</h2>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/health</code> - Health check
        </div>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/version</code> - Version info
        </div>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/docs</code> - This documentation
        </div>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/dashboard/stats</code> - Dashboard overview stats
        </div>
    </div>

    <div class="section">
        <h2>Brain API</h2>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/brain/knowledge</code> - List knowledge slices
        </div>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/brain/health</code> - Brain health check
        </div>
    </div>

    <div class="section">
        <h2>Engine API</h2>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/engine/sessions</code> - List sessions
        </div>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/engine/environments</code> - List environments
        </div>
    </div>

    <div class="section">
        <h2>Synergy API</h2>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/synergy/agents</code> - List agents
        </div>
        <div class="endpoint">
            <span class="method">GET</span> <code>/api/synergy/tasks</code> - List tasks
        </div>
    </div>

    <div class="section">
        <h2>WebSocket Endpoints</h2>
        <div class="endpoint">
            <span class="method">WS</span> <code>/api/ws/sessions/:id</code> - Real-time session status
        </div>
        <div class="endpoint">
            <span class="method">WS</span> <code>/api/ws/missions</code> - Real-time mission updates
        </div>
    </div>
</body>
</html>"#,
        env!("CARGO_PKG_VERSION")
    )
}

// ============================================================================
// Test Helpers
// ============================================================================

pub mod test_helpers {
    use super::*;

    /// Create a test application state
    pub async fn create_test_app_state() -> AppState {
        let session_manager = Arc::new(SessionManager::new());

        let hot = Arc::new(InMemoryHotMemory::new()) as Arc<dyn HotMemoryBackend>;
        let vector =
            Arc::new(InMemoryVectorStore::with_default_config()) as Arc<dyn VectorMemoryBackend>;
        let graph = Arc::new(InMemoryGraphStore::new()) as Arc<dyn GraphMemoryBackend>;

        let raw_dir = std::env::temp_dir().join("agent-playground-test-raw");
        let raw = Arc::new(
            FileSystemRawArchive::new(RawArchiveConfig {
                storage_dir: raw_dir,
                ..Default::default()
            })
            .await
            .unwrap(),
        ) as Arc<dyn RawArchiveBackend>;

        let brain_memory = Arc::new(UnifiedMemory::new(hot, vector, graph, raw));

        AppState {
            config: ServerConfig::default(),
            session_manager,
            brain_memory,
            registry: Arc::new(synergy::registry::AgentRegistry::new()),
            mission_control: Arc::new(synergy::scheduler::MissionControl::new(
                Arc::new(synergy::registry::AgentRegistry::new()),
                synergy::scheduler::SchedulerConfig::default(),
            )),
        }
    }

    /// Create a test router
    pub fn create_test_router(state: AppState) -> Router {
        Router::new()
            .nest("/api", create_api_routes())
            .with_state(state)
    }

    /// Helper function to call the router in tests
    pub async fn call_router(
        router: &mut Router,
        req: axum::extract::Request<Body>,
    ) -> axum::response::Response<Body> {
        use tower::Service;
        Service::call(router, req).await.unwrap()
    }
}

async fn system_websocket(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_system_socket(socket))
}

async fn handle_system_socket(mut socket: WebSocket) {
    info!("WebSocket connected for system monitoring");
    let mut sys = System::new_all();
    let pid = sysinfo::get_current_pid().expect("failed to get PID");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));

    loop {
        interval.tick().await;
        sys.refresh_all();

        let mut cpu_usage = 0.0;
        let mut memory_usage = 0;

        if let Some(process) = sys.process(pid) {
            cpu_usage = process.cpu_usage();
            memory_usage = process.memory(); // in bytes
        }

        let total_memory = sys.total_memory();
        let memory_percent = if total_memory > 0 {
            (memory_usage as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        let msg = json!({
            "type": "system_stats",
            "data": {
                "cpu_usage": cpu_usage,
                "memory_usage_bytes": memory_usage,
                "memory_usage_percent": memory_percent,
                "total_memory_bytes": total_memory,
            }
        });

        if socket.send(Message::Text(msg.to_string())).await.is_err() {
            info!("System WebSocket client disconnected");
            break;
        }
    }
}
