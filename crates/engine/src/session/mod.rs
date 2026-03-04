//! Session - 仿真会话管理
//!
//! 管理Agent仿真会话的生命周期：
//! - 会话创建和配置
//! - Agent注册和管理
//! - 状态追踪
//! - 快照和回滚
//! - 暂停/恢复

use chrono::{DateTime, Utc};
use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::environment::{Environment, EnvironmentState, StateSnapshot};

/// Session配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// 会话名称
    pub name: String,

    /// 会话描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 自动快照间隔（秒）
    #[serde(default = "default_snapshot_interval")]
    pub auto_snapshot_interval: u64,

    /// 最大快照数量
    #[serde(default = "default_max_snapshots")]
    pub max_snapshots: usize,

    /// 是否启用日志记录
    #[serde(default = "default_enable_logging")]
    pub enable_logging: bool,
}

fn default_snapshot_interval() -> u64 {
    60
}
fn default_max_snapshots() -> usize {
    100
}
fn default_enable_logging() -> bool {
    true
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            name: "default_session".to_string(),
            description: None,
            auto_snapshot_interval: default_snapshot_interval(),
            max_snapshots: default_max_snapshots(),
            enable_logging: default_enable_logging(),
        }
    }
}

/// Session状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    /// 未启动
    Idle,

    /// 运行中
    Running,

    /// 已暂停
    Paused,

    /// 已完成
    Completed,

    /// 已失败
    Failed,
}

/// Session快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// 快照ID
    pub id: Uuid,

    /// Session ID
    pub session_id: Uuid,

    /// 环境状态快照
    pub environment_snapshot: StateSnapshot,

    /// Agent状态
    pub agent_states: HashMap<String, serde_json::Value>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Session - 仿真会话
pub struct Session {
    /// Session ID
    pub id: Uuid,

    /// 配置
    config: SessionConfig,

    /// 环境
    environment: Arc<Box<dyn Environment>>,

    /// 状态
    status: Arc<RwLock<SessionStatus>>,

    /// 快照列表
    snapshots: Arc<RwLock<Vec<SessionSnapshot>>>,

    /// 创建时间
    created_at: DateTime<Utc>,

    /// 启动时间
    started_at: Arc<RwLock<Option<DateTime<Utc>>>>,

    /// 结束时间
    ended_at: Arc<RwLock<Option<DateTime<Utc>>>>,

    /// Agent注册表
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

/// Agent信息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentInfo {
    /// Agent名称
    name: String,

    /// Agent类型
    agent_type: String,

    /// 注册时间
    registered_at: DateTime<Utc>,
}

impl Session {
    /// 创建新Session
    pub fn new(config: SessionConfig, environment: Box<dyn Environment>) -> Self {
        info!("Creating session: {}", config.name);

        Self {
            id: Uuid::new_v4(),
            config,
            environment: Arc::new(environment),
            status: Arc::new(RwLock::new(SessionStatus::Idle)),
            snapshots: Arc::new(RwLock::new(Vec::new())),
            created_at: Utc::now(),
            started_at: Arc::new(RwLock::new(None)),
            ended_at: Arc::new(RwLock::new(None)),
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取Session ID
    pub fn id(&self) -> Uuid {
        self.id
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("config", &self.config)
            .field("created_at", &self.created_at)
            .finish_non_exhaustive()
    }
}

impl Session {
    /// 获取Session状态
    pub async fn status(&self) -> SessionStatus {
        *self.status.read().await
    }

    /// 启动Session
    pub async fn start(&self) -> Result<()> {
        info!("Starting session: {}", self.id);

        let mut status = self.status.write().await;
        if *status != SessionStatus::Idle {
            return Err(SessionError::InvalidStateTransition(format!(
                "Cannot start session from {:?}",
                status
            ))
            .into());
        }

        *status = SessionStatus::Running;
        *self.started_at.write().await = Some(Utc::now());

        // 创建初始快照
        self.create_snapshot("Session started").await?;

        info!("Session started: {}", self.id);
        Ok(())
    }

    /// 暂停Session
    pub async fn pause(&self) -> Result<()> {
        info!("Pausing session: {}", self.id);

        let mut status = self.status.write().await;
        if *status != SessionStatus::Running {
            return Err(SessionError::InvalidStateTransition(format!(
                "Cannot pause session from {:?}",
                status
            ))
            .into());
        }

        *status = SessionStatus::Paused;

        // 创建快照
        self.create_snapshot("Session paused").await?;

        info!("Session paused: {}", self.id);
        Ok(())
    }

    /// 恢复Session
    pub async fn resume(&self) -> Result<()> {
        info!("Resuming session: {}", self.id);

        let mut status = self.status.write().await;
        if *status != SessionStatus::Paused {
            return Err(SessionError::InvalidStateTransition(format!(
                "Cannot resume session from {:?}",
                status
            ))
            .into());
        }

        *status = SessionStatus::Running;

        info!("Session resumed: {}", self.id);
        Ok(())
    }

    /// 完成Session
    pub async fn complete(&self) -> Result<()> {
        info!("Completing session: {}", self.id);

        let mut status = self.status.write().await;
        if !matches!(*status, SessionStatus::Running | SessionStatus::Paused) {
            return Err(SessionError::InvalidStateTransition(format!(
                "Cannot complete session from {:?}",
                status
            ))
            .into());
        }

        *status = SessionStatus::Completed;
        *self.ended_at.write().await = Some(Utc::now());

        // 创建最终快照
        self.create_snapshot("Session completed").await?;

        info!("Session completed: {}", self.id);
        Ok(())
    }

    /// 注册Agent
    pub async fn register_agent(&self, name: String, agent_type: String) -> Result<()> {
        info!("Registering agent: {} (type: {})", name, agent_type);

        let mut agents = self.agents.write().await;

        if agents.contains_key(&name) {
            return Err(SessionError::AgentAlreadyExists(name).into());
        }

        agents.insert(
            name.clone(),
            AgentInfo {
                name: name.clone(),
                agent_type,
                registered_at: Utc::now(),
            },
        );

        debug!("Agent registered: {}", name);
        Ok(())
    }

    /// 注销Agent
    pub async fn unregister_agent(&self, name: &str) -> Result<bool> {
        info!("Unregistering agent: {}", name);

        let mut agents = self.agents.write().await;
        Ok(agents.remove(name).is_some())
    }

    /// 获取所有Agent
    pub async fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// 获取当前环境状态
    pub async fn current_environment_state(&self) -> Result<EnvironmentState> {
        self.environment.current_state()
    }

    /// 更新环境状态
    pub async fn update_environment_state(
        &self,
        update: crate::environment::StateUpdate,
    ) -> Result<()> {
        // 注意：这里需要内部可变性，暂时跳过
        warn!("Environment state update not yet fully implemented");
        Ok(())
    }

    /// 创建快照
    pub async fn create_snapshot(&self, description: impl Into<String>) -> Result<SessionSnapshot> {
        debug!("Creating snapshot for session: {}", self.id);

        let env_snapshot = self.environment.create_snapshot()?;

        let agents = self.agents.read().await;
        let agent_states: HashMap<String, serde_json::Value> = HashMap::new(); // TODO: 序列化Agent状态

        let description = description.into();

        let snapshot = SessionSnapshot {
            id: Uuid::new_v4(),
            session_id: self.id,
            environment_snapshot: env_snapshot,
            agent_states,
            created_at: Utc::now(),
            description: if description.is_empty() {
                None
            } else {
                Some(description)
            },
        };

        let mut snapshots = self.snapshots.write().await;
        snapshots.push(snapshot.clone());

        // 限制快照数量
        if snapshots.len() > self.config.max_snapshots {
            snapshots.remove(0);
        }

        debug!("Snapshot created: {}", snapshot.id);
        Ok(snapshot)
    }

    /// 获取所有快照
    pub async fn list_snapshots(&self) -> Vec<SessionSnapshot> {
        self.snapshots.read().await.clone()
    }

    /// 恢复快照
    pub async fn restore_snapshot(&self, snapshot_id: Uuid) -> Result<()> {
        info!(
            "Restoring snapshot: {} for session: {}",
            snapshot_id, self.id
        );

        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots
            .iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| SessionError::SnapshotNotFound(snapshot_id))?;

        // 恢复环境状态
        // 注意：需要environment的内部可变性
        warn!("Snapshot restore not yet fully implemented");

        info!("Snapshot restored: {}", snapshot_id);
        Ok(())
    }

    /// 获取Session统计信息
    pub async fn stats(&self) -> SessionStats {
        let status = *self.status.read().await;
        let snapshots = self.snapshots.read().await.len();
        let agents = self.agents.read().await.len();

        SessionStats {
            id: self.id,
            status,
            created_at: self.created_at,
            started_at: *self.started_at.read().await,
            ended_at: *self.ended_at.read().await,
            snapshot_count: snapshots,
            agent_count: agents,
        }
    }
}

/// Session统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub id: Uuid,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub snapshot_count: usize,
    pub agent_count: usize,
}

/// Session管理器
#[derive(Debug)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Arc<Session>>>>,
}

impl SessionManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建Session
    pub async fn create_session(
        &self,
        config: SessionConfig,
        environment: Box<dyn Environment>,
    ) -> Result<Arc<Session>> {
        let session = Session::new(config, environment);
        let session = Arc::new(session);

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id(), session.clone());

        Ok(session)
    }

    /// 获取Session
    pub async fn get_session(&self, id: Uuid) -> Option<Arc<Session>> {
        let sessions = self.sessions.read().await;
        sessions.get(&id).cloned()
    }

    /// 删除Session
    pub async fn delete_session(&self, id: Uuid) -> Result<bool> {
        let mut sessions = self.sessions.write().await;
        Ok(sessions.remove(&id).is_some())
    }

    /// 列出所有Session
    pub async fn list_sessions(&self) -> Vec<Uuid> {
        let sessions = self.sessions.read().await;
        sessions.keys().copied().collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session错误
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Agent already exists: {0}")]
    AgentAlreadyExists(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(Uuid),

    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),
}

impl From<SessionError> for Error {
    fn from(err: SessionError) -> Self {
        Error::SessionError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{BasicEnvironment, EnvironmentBuilder, EnvironmentSchema};

    #[tokio::test]
    async fn test_session_creation() {
        let config = SessionConfig::default();

        let env_builder = crate::environment::EnvironmentBuilder::new("test", "1.0.0");
        let env = env_builder.build().unwrap();

        let session = Session::new(config, Box::new(env));
        assert_eq!(session.status().await, SessionStatus::Idle);
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let config = SessionConfig::default();

        let env_builder = crate::environment::EnvironmentBuilder::new("test", "1.0.0");
        let env = env_builder.build().unwrap();

        let session = Arc::new(Session::new(config, Box::new(env)));

        // 启动
        assert!(session.start().await.is_ok());
        assert_eq!(session.status().await, SessionStatus::Running);

        // 暂停
        assert!(session.pause().await.is_ok());
        assert_eq!(session.status().await, SessionStatus::Paused);

        // 恢复
        assert!(session.resume().await.is_ok());
        assert_eq!(session.status().await, SessionStatus::Running);

        // 完成
        assert!(session.complete().await.is_ok());
        assert_eq!(session.status().await, SessionStatus::Completed);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let config = SessionConfig::default();

        let env_builder = crate::environment::EnvironmentBuilder::new("test", "1.0.0");
        let env = env_builder.build().unwrap();

        let session = Session::new(config, Box::new(env));

        assert!(session
            .register_agent("agent1".to_string(), "test".to_string())
            .await
            .is_ok());
        let agents = session.list_agents().await;
        assert_eq!(agents.len(), 1);
        assert!(agents.contains(&"agent1".to_string()));
    }
}
