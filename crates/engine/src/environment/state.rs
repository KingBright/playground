//! Environment State - 状态管理和快照
//!
//! 提供状态管理、验证、快照和回滚功能

use chrono::{DateTime, Utc};
use common::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Environment状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    /// 状态数据
    pub data: HashMap<String, Value>,

    /// 状态版本号
    pub version: u64,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 状态元数据
    pub metadata: HashMap<String, String>,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            data: HashMap::new(),
            version: 0,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }
}

impl EnvironmentState {
    /// 创建新状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 从数据创建状态
    pub fn from_data(data: HashMap<String, Value>) -> Self {
        let now = Utc::now();
        Self {
            data,
            version: 0,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// 获取字段值
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// 设置字段值
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.data.insert(key.into(), value);
        self.version += 1;
        self.updated_at = Utc::now();
    }

    /// 检查是否包含字段
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// 获取所有键
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    /// 序列化为JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.data).map_err(|e| Error::SerializationError(e.into()))
    }

    /// 从JSON反序列化
    pub fn from_json(json: &str) -> Result<Self> {
        let data: HashMap<String, Value> =
            serde_json::from_str(json).map_err(|e| Error::SerializationError(e.into()))?;

        Ok(Self::from_data(data))
    }
}

/// 状态更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    /// 更新的字段
    pub changes: HashMap<String, Value>,

    /// 更新元数据
    pub metadata: HashMap<String, String>,

    /// 更新时间
    pub timestamp: DateTime<Utc>,
}

impl StateUpdate {
    /// 创建新的状态更新
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// 设置字段值
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.changes.insert(key.into(), value);
    }

    /// 批量设置
    pub fn set_all(&mut self, changes: HashMap<String, Value>) {
        self.changes = changes;
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

impl Default for StateUpdate {
    fn default() -> Self {
        Self::new()
    }
}

/// 状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// 快照ID
    pub id: Uuid,

    /// 环境名称
    pub environment_name: String,

    /// 状态数据
    pub state: EnvironmentState,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl StateSnapshot {
    /// 创建新快照
    pub fn new(environment_name: String, state: EnvironmentState) -> Self {
        Self {
            id: Uuid::new_v4(),
            environment_name,
            state,
            created_at: Utc::now(),
        }
    }
}

/// 状态转换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// 转换ID
    pub id: Uuid,

    /// 从状态
    pub from_state: EnvironmentState,

    /// 到状态
    pub to_state: EnvironmentState,

    /// 更新
    pub update: StateUpdate,

    /// 转换时间
    pub timestamp: DateTime<Utc>,

    /// 转换结果
    pub success: bool,

    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl StateTransition {
    /// 创建新的状态转换
    pub fn new(from_state: EnvironmentState, update: StateUpdate) -> Self {
        let to_state = from_state.clone(); // TODO: 应用更新
        Self {
            id: Uuid::new_v4(),
            from_state,
            to_state,
            update,
            timestamp: Utc::now(),
            success: true,
            error: None,
        }
    }

    /// 标记为失败
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error = Some(error.into());
        self
    }
}

/// 状态验证器
pub trait StateValidator: Send + Sync {
    /// 验证状态
    fn validate(&self, state: &EnvironmentState) -> Result<bool>;

    /// 验证转换
    fn validate_transition(&self, from: &EnvironmentState, to: &EnvironmentState) -> Result<bool> {
        // 默认只验证目标状态
        self.validate(to)
    }
}

/// 默认验证器
#[derive(Debug, Default)]
pub struct DefaultValidator;

impl StateValidator for DefaultValidator {
    fn validate(&self, _state: &EnvironmentState) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = EnvironmentState::new();
        assert_eq!(state.version, 0);
        assert!(state.data.is_empty());
    }

    #[test]
    fn test_state_operations() {
        let mut state = EnvironmentState::new();

        state.set("key1", serde_json::json!("value1"));
        assert_eq!(state.get("key1"), Some(&serde_json::json!("value1")));
        assert!(state.contains_key("key1"));
        assert_eq!(state.version, 1);
    }

    #[test]
    fn test_state_serialization() {
        let mut state = EnvironmentState::new();
        state.set("key", serde_json::json!("value"));

        let json = state.to_json();
        assert!(json.is_ok());

        let restored = EnvironmentState::from_json(&json.unwrap());
        assert!(restored.is_ok());
        assert_eq!(
            restored.unwrap().get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_state_update() {
        let mut update = StateUpdate::new();
        update.set("key1", serde_json::json!("value1"));
        update.set("key2", serde_json::json!("value2"));

        assert_eq!(update.changes.len(), 2);
    }

    #[test]
    fn test_snapshot() {
        let state = EnvironmentState::new();
        let snapshot = StateSnapshot::new("test_env".to_string(), state);

        assert_eq!(snapshot.environment_name, "test_env");
    }

    #[test]
    fn test_default_validator() {
        let validator = DefaultValidator;
        let state = EnvironmentState::new();

        assert!(validator.validate(&state).is_ok());
    }
}
