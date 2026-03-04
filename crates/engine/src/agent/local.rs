//! Local Agent - 环境原生Agent
//!
//! 为特定Environment定制的Agent实现

use crate::agent::{AgentAction, AgentBehavior, AgentConfig, AgentContext, AgentResult};
use crate::environment::{Environment, EnvironmentState};
use common::{Agent, AgentCapabilities, AgentInput, AgentOutput, Error, Result};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Local Agent配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalAgentConfig {
    /// Agent名称
    pub name: String,

    /// Agent描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Agent角色/身份
    pub role: String,

    /// Agent特性
    pub characteristics: Vec<String>,

    /// 行为脚本 (Rhai)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior_script: Option<String>,
}

impl Default for LocalAgentConfig {
    fn default() -> Self {
        Self {
            name: "agent".to_string(),
            description: None,
            role: "generic".to_string(),
            characteristics: vec![],
            behavior_script: None,
        }
    }
}

/// Local Agent实现
#[derive(Debug)]
pub struct LocalAgent {
    /// Agent配置
    config: LocalAgentConfig,

    /// Agent ID
    id: Uuid,

    /// 当前状态
    state: Arc<RwLock<LocalAgentState>>,
}

/// Local Agent内部状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LocalAgentState {
    /// 当前位置/状态
    pub position: serde_json::Value,

    /// Agent记忆
    pub memory: Vec<serde_json::Value>,

    /// Agent统计
    pub stats: AgentStats,
}

/// Agent统计信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentStats {
    /// 采取的行动数
    pub actions_taken: u64,

    /// 获得的奖励数
    pub rewards_earned: f64,

    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for LocalAgentState {
    fn default() -> Self {
        Self {
            position: json!({}),
            memory: vec![],
            stats: AgentStats {
                actions_taken: 0,
                rewards_earned: 0.0,
                last_updated: chrono::Utc::now(),
            },
        }
    }
}

impl LocalAgent {
    /// 创建新的Local Agent
    pub fn new(config: LocalAgentConfig) -> Self {
        info!(
            "Creating local agent: {} (role: {})",
            config.name, config.role
        );

        Self {
            id: Uuid::new_v4(),
            config,
            state: Arc::new(RwLock::new(LocalAgentState::default())),
        }
    }

    /// 获取Agent ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// 获取Agent名称
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// 获取Agent角色
    pub fn role(&self) -> &str {
        &self.config.role
    }

    /// 更新Agent状态
    pub async fn update_state(&self, updates: serde_json::Value) -> Result<()> {
        let mut state = self.state.write().await;

        if let Some(obj) = updates.as_object() {
            for (key, value) in obj {
                state.position[key] = value.clone();
            }
        }

        state.stats.last_updated = chrono::Utc::now();
        debug!("Agent {} state updated", self.config.name);
        Ok(())
    }

    /// 记录记忆
    pub async fn remember(&self, memory: serde_json::Value) -> Result<()> {
        let mut state = self.state.write().await;
        state.memory.push(memory);
        debug!("Agent {} remembered new information", self.config.name);
        Ok(())
    }

    /// 执行行为
    pub async fn execute_behavior(
        &self,
        environment_state: &EnvironmentState,
    ) -> Result<AgentAction> {
        // 如果有行为脚本，使用脚本
        if let Some(script) = &self.config.behavior_script {
            return self
                .execute_script_behavior(script, environment_state)
                .await;
        }

        // 默认行为
        self.default_behavior(environment_state).await
    }

    /// 执行脚本行为
    async fn execute_script_behavior(
        &self,
        _script: &str,
        _environment_state: &EnvironmentState,
    ) -> Result<AgentAction> {
        // 简化实现：不执行脚本，返回默认行为
        // 在实际使用中，可以使用rhai引擎执行
        self.default_behavior(_environment_state).await
    }

    /// 默认行为（简单随机行为）
    async fn default_behavior(&self, _environment_state: &EnvironmentState) -> Result<AgentAction> {
        Ok(AgentAction {
            action_type: "wait".to_string(),
            parameters: json!({
                "duration_ms": 1000
            }),
        })
    }
}

#[async_trait::async_trait]
impl Agent for LocalAgent {
    async fn invoke(&self, input: AgentInput) -> Result<AgentOutput> {
        debug!("LocalAgent {} invoked", self.config.name);

        // 获取环境状态
        let environment_state_json = input
            .data
            .get("environment_state")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let environment_state =
            EnvironmentState::from_data(environment_state_json.into_iter().collect());

        // 执行行为
        let action = self.execute_behavior(&environment_state).await?;

        // 更新统计
        {
            let mut state = self.state.write().await;
            state.stats.actions_taken += 1;
        }

        // 构建输出
        let output_data = json!({
            "agent_id": self.id,
            "agent_name": self.config.name,
            "action": action,
            "role": self.config.role,
            "timestamp": chrono::Utc::now(),
        });

        Ok(AgentOutput {
            data: output_data,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("agent_type".to_string(), "local".to_string());
                meta
            },
            need_help: None,
        })
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn capabilities(&self) -> AgentCapabilities {
        vec![
            "environment_interaction".to_string(),
            format!("role:{}", self.config.role),
            "state_management".to_string(),
            "memory".to_string(),
        ]
    }
}

/// 创建Chess Player Agent
pub fn create_chess_player(name: String, color: String) -> LocalAgent {
    LocalAgentConfig {
        name,
        description: Some(format!("Chess player - {}", color)),
        role: "chess_player".to_string(),
        characteristics: vec!["strategic".to_string(), "competitive".to_string()],
        behavior_script: Some(format!(
            r#"
        // Chess player behavior
        let board = env_state.get("board");
        let my_color = "{}";

        // 简单决策：随机选择一个合法移动
        let action_type = "move_piece";

        let parameters = {{
            "from": "random",
            "to": "random",
            "piece": "pawn"
        }};

        {{ action_type, parameters }}
        "#,
            color
        )),
    }
    .into()
}

/// 创建Debate Participant Agent
pub fn create_debate_participant(name: String, position: String) -> LocalAgent {
    LocalAgentConfig {
        name,
        description: Some(format!("Debate participant - {}", position)),
        role: "debater".to_string(),
        characteristics: vec!["articulate".to_string(), "persuasive".to_string()],
        behavior_script: Some(format!(
            r#"
        // Debate participant behavior
        let topic = env_state.get("topic");
        let my_position = "{}";

        // 构建论点
        let action_type = "present_argument";

        let parameters = {{
            "argument": format!("I believe in {{}} because it is the right approach.", my_position),
            "evidence": ["fact1", "fact2"],
            "confidence": 0.8
        }};

        {{ action_type, parameters }}
        "#,
            position
        )),
    }
    .into()
}

impl From<LocalAgentConfig> for LocalAgent {
    fn from(config: LocalAgentConfig) -> Self {
        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_agent_creation() {
        let config = LocalAgentConfig {
            name: "test_agent".to_string(),
            ..Default::default()
        };

        let agent = LocalAgent::new(config);
        assert_eq!(agent.name(), "test_agent");
        assert_eq!(agent.role(), "generic");
    }

    #[tokio::test]
    async fn test_agent_state_update() {
        let config = LocalAgentConfig::default();
        let agent = LocalAgent::new(config);

        let updates = json!({
            "x": 10,
            "y": 20
        });

        assert!(agent.update_state(updates).await.is_ok());

        let state = agent.state.read().await;
        assert_eq!(state.position["x"], 10);
        assert_eq!(state.position["y"], 20);
    }

    #[tokio::test]
    async fn test_agent_memory() {
        let config = LocalAgentConfig::default();
        let agent = LocalAgent::new(config);

        let memory = json!({
            "event": "something happened",
            "timestamp": "2024-01-01"
        });

        assert!(agent.remember(memory).await.is_ok());

        let state = agent.state.read().await;
        assert_eq!(state.memory.len(), 1);
    }

    #[test]
    fn test_chess_player_creation() {
        let player = create_chess_player("Alice".to_string(), "white".to_string());
        assert_eq!(player.name(), "Alice");
        assert_eq!(player.role(), "chess_player");
    }

    #[test]
    fn test_debate_participant_creation() {
        let debater = create_debate_participant("Bob".to_string(), "pro".to_string());
        assert_eq!(debater.name(), "Bob");
        assert_eq!(debater.role(), "debater");
    }

    #[tokio::test]
    async fn test_agent_invoke() {
        let config = LocalAgentConfig {
            name: "test".to_string(),
            ..Default::default()
        };

        let agent = LocalAgent::new(config);

        let input = AgentInput::new(json!({
            "environment_state": {
                "board": "initial_state"
            }
        }));

        let result = agent.invoke(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.data["agent_name"], "test");
    }
}
