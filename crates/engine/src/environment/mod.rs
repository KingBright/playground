//! Environment - 仿真环境定义和管理
//!
//! 提供可自定义的仿真环境：
//! - 状态定义和验证
//! - 动态渲染脚本
//! - 状态快照和回滚

pub mod examples;
pub mod schema;
pub mod state;

pub use schema::{EnvironmentError, EnvironmentSchema, RendererScript, StateSchema, ValidatorDef};

pub use state::{EnvironmentState, StateSnapshot, StateTransition, StateUpdate, StateValidator};

use common::{Error, Result};
use examples::{ChessEnvironment, DebateEnvironment};

/// Environment trait - 所有环境的基础接口
pub trait Environment: Send + Sync {
    /// 获取环境名称
    fn name(&self) -> &str;

    /// 获取环境版本
    fn version(&self) -> &str;

    /// 获取当前状态
    fn current_state(&self) -> Result<EnvironmentState>;

    /// 更新状态
    fn update_state(&mut self, update: StateUpdate) -> Result<()>;

    /// 验证状态
    fn validate_state(&self, state: &EnvironmentState) -> Result<bool>;

    /// 渲染状态为可读格式
    fn render_state(&self, state: &EnvironmentState) -> Result<String>;

    /// 创建快照
    fn create_snapshot(&self) -> Result<StateSnapshot>;

    /// 恢复快照
    fn restore_snapshot(&mut self, snapshot: &StateSnapshot) -> Result<()>;

    /// 重置环境到初始状态
    fn reset(&mut self) -> Result<()>;
}

/// Environment构建器
pub struct EnvironmentBuilder {
    name: String,
    version: String,
    schema: EnvironmentSchema,
}

impl EnvironmentBuilder {
    /// 创建新的构建器
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            schema: EnvironmentSchema::default(),
        }
    }

    /// 设置schema
    pub fn with_schema(mut self, schema: EnvironmentSchema) -> Self {
        self.schema = schema;
        self
    }

    /// 构建（返回基础实现）
    pub fn build(self) -> Result<impl Environment> {
        // 这里返回一个基础实现
        Ok(BasicEnvironment {
            name: self.name,
            version: self.version,
            state: EnvironmentState::default(),
            schema: self.schema,
        })
    }
}

/// 基础Environment实现
#[derive(Debug)]
pub struct BasicEnvironment {
    name: String,
    version: String,
    state: EnvironmentState,
    schema: EnvironmentSchema,
}

impl Environment for BasicEnvironment {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn current_state(&self) -> Result<EnvironmentState> {
        Ok(self.state.clone())
    }

    fn update_state(&mut self, update: StateUpdate) -> Result<()> {
        // 验证更新
        if let Some(validator) = &self.schema.validator {
            if !validator.validate(&self.state, &update)? {
                return Err(Error::EnvironmentError(
                    "State update validation failed".to_string(),
                ));
            }
        }

        // 应用更新
        for (key, value) in update.changes {
            self.state.data.insert(key, value);
        }

        self.state.version += 1;
        self.state.updated_at = chrono::Utc::now();

        Ok(())
    }

    fn validate_state(&self, state: &EnvironmentState) -> Result<bool> {
        if let Some(validator) = &self.schema.validator {
            validator.validate_state(state)
        } else {
            Ok(true)
        }
    }

    fn render_state(&self, state: &EnvironmentState) -> Result<String> {
        if let Some(renderer) = &self.schema.renderer {
            renderer.render(state)
        } else {
            // 默认JSON渲染
            Ok(serde_json::to_string_pretty(&state.data)?)
        }
    }

    fn create_snapshot(&self) -> Result<StateSnapshot> {
        Ok(StateSnapshot {
            id: uuid::Uuid::new_v4(),
            environment_name: self.name.clone(),
            state: self.state.clone(),
            created_at: chrono::Utc::now(),
        })
    }

    fn restore_snapshot(&mut self, snapshot: &StateSnapshot) -> Result<()> {
        if snapshot.environment_name != self.name {
            return Err(Error::EnvironmentError(format!(
                "Snapshot is for {}, current is {}",
                snapshot.environment_name, self.name
            )));
        }

        self.state = snapshot.state.clone();
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.state = EnvironmentState::default();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_environment_builder() {
        let schema = EnvironmentSchema::default();
        let env = EnvironmentBuilder::new("test", "1.0.0")
            .with_schema(schema)
            .build();

        assert!(env.is_ok());
        let env = env.unwrap();
        assert_eq!(env.name(), "test");
        assert_eq!(env.version(), "1.0.0");
    }

    #[test]
    fn test_state_update() {
        let mut env = BasicEnvironment {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            state: EnvironmentState::default(),
            schema: EnvironmentSchema::default(),
        };

        let mut update = StateUpdate::new();
        update.set("key1", json!("value1"));

        assert!(env.update_state(update).is_ok());
        let state = env.current_state().unwrap();
        assert_eq!(state.data.get("key1"), Some(&json!("value1")));
    }
}
