//! Agent Registry - Agent注册和管理
//!
//! 提供Agent的动态注册和实例化

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, Row};

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
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    /// 存储的Agent定义 (In-memory cache)
    agents: Arc<RwLock<HashMap<String, AgentDefinition>>>,
    /// SQLite Database pool for persistence
    db: Option<SqlitePool>,
}

impl AgentRegistry {
    /// 创建新的Registry (with persistent DB if URL provided, or fallback to memory)
    pub async fn new(db_url: Option<&str>) -> Self {
        info!("Creating agent registry");

        let mut db_pool = None;
        if let Some(url) = db_url {
             match SqlitePoolOptions::new()
                .max_connections(5)
                .connect(url).await
            {
                Ok(pool) => {
                    info!("Connected to SQLite database at {}", url);

                    // Initialize schema
                    let init_result = sqlx::query(
                        "CREATE TABLE IF NOT EXISTS agents (
                            id TEXT PRIMARY KEY,
                            name TEXT UNIQUE NOT NULL,
                            agent_type TEXT NOT NULL,
                            config TEXT NOT NULL,
                            description TEXT,
                            created_at DATETIME NOT NULL
                        );"
                    ).execute(&pool).await;

                    if let Err(e) = init_result {
                        error!("Failed to initialize database schema: {}", e);
                    } else {
                        db_pool = Some(pool);
                    }
                },
                Err(e) => error!("Failed to connect to SQLite: {}. Falling back to in-memory.", e),
            }
        }

        let registry = Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            db: db_pool,
        };

        // Load existing agents from DB into cache
        if let Some(ref pool) = registry.db {
            if let Ok(rows) = sqlx::query("SELECT * FROM agents").fetch_all(pool).await {
                let mut cache = registry.agents.write().await;
                for row in rows {
                    let id_str: String = row.get("id");
                    let name: String = row.get("name");
                    let agent_type_str: String = row.get("agent_type");
                    let config_str: String = row.get("config");
                    let description: Option<String> = row.get("description");

                    let id = uuid::Uuid::parse_str(&id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
                    let agent_type = match agent_type_str.as_str() {
                        "Local" => AgentType::Local,
                        _ => AgentType::Universal,
                    };
                    let config = serde_json::from_str(&config_str).unwrap_or_else(|_| serde_json::json!({}));

                    let def = AgentDefinition {
                        id,
                        name: name.clone(),
                        agent_type,
                        config,
                        description,
                        created_at: chrono::Utc::now(), // Simplified for now
                    };
                    cache.insert(name, def);
                }
            }
        }

        registry
    }

    /// 注册Agent
    pub async fn register(&self, definition: AgentDefinition) -> Result<()> {
        info!("Registering agent: {}", definition.name);

        // Persist to DB if available
        if let Some(ref pool) = self.db {
            let id_str = definition.id.to_string();
            let type_str = match definition.agent_type {
                AgentType::Local => "Local",
                AgentType::Universal => "Universal",
            };
            let config_str = definition.config.to_string();

            let query_result = sqlx::query(
                "INSERT INTO agents (id, name, agent_type, config, description, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)
                 ON CONFLICT(name) DO UPDATE SET
                    agent_type=excluded.agent_type,
                    config=excluded.config,
                    description=excluded.description"
            )
            .bind(id_str)
            .bind(&definition.name)
            .bind(type_str)
            .bind(config_str)
            .bind(&definition.description)
            .bind(definition.created_at.timestamp())
            .execute(pool)
            .await;

            if let Err(e) = query_result {
                error!("Failed to persist agent to DB: {}", e);
                return Err(Error::Internal(format!("DB Error: {}", e)));
            }
        }

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

        // Remove from DB if available
        if let Some(ref pool) = self.db {
            if let Err(e) = sqlx::query("DELETE FROM agents WHERE name = ?")
                .bind(name)
                .execute(pool)
                .await
            {
                error!("Failed to delete agent from DB: {}", e);
            }
        }

        let mut agents = self.agents.write().await;
        agents.remove(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = AgentRegistry::new(None).await;
        assert_eq!(registry.list().await.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let registry = AgentRegistry::new(None).await;

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
        let registry = AgentRegistry::new(None).await;

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
