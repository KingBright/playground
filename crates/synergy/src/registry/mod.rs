//! Agent Registry - Agent注册和管理
//!
//! 提供Agent的动态注册和实例化

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Agent类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Local,
    Universal,
}

/// Agent定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Agent ID
    pub id: uuid::Uuid,

    /// Agent名称
    pub name: String,

    /// Agent类型
    pub agent_type: AgentType,

    /// Agent配置
    pub config: serde_json::Value,

    /// Agent描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Agent Registry
#[derive(Debug)]
pub struct AgentRegistry {
    /// 存储的Agent定义
    agents: Arc<RwLock<HashMap<String, AgentDefinition>>>,
}

impl AgentRegistry {
    /// 创建新的Registry
    pub fn new() -> Self {
        info!("Creating agent registry");

        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册Agent
    pub async fn register(&self, definition: AgentDefinition) -> Result<()> {
        info!("Registering agent: {}", definition.name);

        let mut agents = self.agents.write().await;
        agents.insert(definition.name.clone(), definition);

        Ok(())
    }

    /// 获取Agent定义
    pub async fn get(&self, name: &str) -> Option<AgentDefinition> {
        let agents = self.agents.read().await;
        agents.get(name).cloned()
    }

    /// 列出所有Agent
    pub async fn list(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// 注销Agent
    pub async fn unregister(&self, name: &str) -> bool {
        info!("Unregistering agent: {}", name);

        let mut agents = self.agents.write().await;
        agents.remove(name).is_some()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.list().await.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let registry = AgentRegistry::new();

        let definition = AgentDefinition {
            id: uuid::Uuid::new_v4(),
            name: "test_agent".to_string(),
            agent_type: AgentType::Local,
            config: serde_json::json!({}),
            description: None,
            created_at: chrono::Utc::now(),
        };

        assert!(registry.register(definition).await.is_ok());
        assert_eq!(registry.list().await.len(), 1);
    }

    #[tokio::test]
    async fn test_agent_retrieval() {
        let registry = AgentRegistry::new();

        let definition = AgentDefinition {
            id: uuid::Uuid::new_v4(),
            name: "test_agent".to_string(),
            agent_type: AgentType::Local,
            config: serde_json::json!({}),
            description: None,
            created_at: chrono::Utc::now(),
        };

        registry.register(definition).await.unwrap();

        let retrieved = registry.get("test_agent").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_agent");
    }
}
