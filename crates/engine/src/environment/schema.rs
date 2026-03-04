//! Environment Schema - 环境定义和验证
//!
//! 定义环境的结构、规则和渲染方式

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Environment Schema定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSchema {
    /// Schema名称
    pub name: String,

    /// Schema版本
    pub version: String,

    /// 状态定义
    pub state_definition: StateSchema,

    /// 验证器
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator: Option<ValidatorDef>,

    /// 渲染器脚本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderer: Option<RendererScript>,
}

impl Default for EnvironmentSchema {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            version: "1.0.0".to_string(),
            state_definition: StateSchema::default(),
            validator: None,
            renderer: None,
        }
    }
}

/// 状态Schema定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSchema {
    /// 必需字段
    pub required_fields: Vec<String>,

    /// 字段类型定义
    pub field_types: HashMap<String, String>,

    /// 字段约束
    pub constraints: HashMap<String, FieldConstraint>,
}

impl Default for StateSchema {
    fn default() -> Self {
        Self {
            required_fields: vec![],
            field_types: HashMap::new(),
            constraints: HashMap::new(),
        }
    }
}

/// 字段约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConstraint {
    /// 最小值
    pub min: Option<f64>,

    /// 最大值
    pub max: Option<f64>,

    /// 枚举值
    pub enum_values: Option<Vec<Value>>,

    /// 正则表达式
    pub pattern: Option<String>,
}

/// 验证器定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidatorDef {
    /// 内置验证器
    BuiltIn(BuiltInValidator),

    /// 自定义验证脚本 (Rhai)
    Custom(String),
}

impl ValidatorDef {
    /// 验证状态转换
    pub fn validate(
        &self,
        old_state: &crate::environment::EnvironmentState,
        update: &crate::environment::StateUpdate,
    ) -> Result<bool> {
        match self {
            ValidatorDef::BuiltIn(validator) => validator.validate(old_state, update),
            ValidatorDef::Custom(script) => {
                // 使用Rhai执行自定义验证
                warn!("Custom validator not yet implemented: {}", script);
                Ok(true)
            }
        }
    }

    /// 验证状态
    pub fn validate_state(&self, state: &crate::environment::EnvironmentState) -> Result<bool> {
        match self {
            ValidatorDef::BuiltIn(validator) => validator.validate_state(state),
            ValidatorDef::Custom(script) => {
                warn!("Custom validator not yet implemented: {}", script);
                Ok(true)
            }
        }
    }
}

/// 内置验证器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuiltInValidator {
    /// 无验证
    None,

    /// 非空验证
    NonEmpty,

    /// 类型验证
    TypeCheck,

    /// 范围验证
    RangeCheck,
}

impl BuiltInValidator {
    pub fn validate(
        &self,
        _old_state: &crate::environment::EnvironmentState,
        _update: &crate::environment::StateUpdate,
    ) -> Result<bool> {
        match self {
            BuiltInValidator::None => Ok(true),
            BuiltInValidator::NonEmpty => {
                // 检查更新是否为空
                Ok(!_update.changes.is_empty())
            }
            BuiltInValidator::TypeCheck => {
                // TODO: 实现类型检查
                Ok(true)
            }
            BuiltInValidator::RangeCheck => {
                // TODO: 实现范围检查
                Ok(true)
            }
        }
    }

    pub fn validate_state(&self, state: &crate::environment::EnvironmentState) -> Result<bool> {
        match self {
            BuiltInValidator::None => Ok(true),
            BuiltInValidator::NonEmpty => Ok(!state.data.is_empty()),
            BuiltInValidator::TypeCheck => Ok(true),
            BuiltInValidator::RangeCheck => Ok(true),
        }
    }
}

/// 渲染器脚本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererScript {
    /// 脚本语言 (目前只支持Rhai)
    pub language: String,

    /// 脚本内容
    pub script: String,
}

impl RendererScript {
    /// 渲染状态
    pub fn render(&self, state: &crate::environment::EnvironmentState) -> Result<String> {
        match self.language.as_str() {
            "rhai" => {
                let engine = rhai::Engine::new();
                let mut scope = rhai::Scope::new();

                // 将state数据注入到scope
                for (key, value) in &state.data {
                    let value_str = serde_json::to_string(value).unwrap_or_default();
                    scope.push_constant(key.clone(), value_str);
                }

                // 执行渲染脚本
                let result = engine
                    .eval_with_scope::<String>(&mut scope, &self.script)
                    .map_err(|e| {
                        Error::WorkflowError(format!("Renderer execution failed: {}", e))
                    })?;

                Ok(result)
            }
            _ => Err(Error::WorkflowError(format!(
                "Unsupported renderer language: {}",
                self.language
            ))),
        }
    }
}

/// Environment错误类型
#[derive(Debug, thiserror::Error)]
pub enum EnvironmentError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Mismatched environment: {0}")]
    MismatchedEnvironment(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<EnvironmentError> for Error {
    fn from(err: EnvironmentError) -> Self {
        Error::EngineError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{EnvironmentState, StateUpdate};

    #[test]
    fn test_schema_default() {
        let schema = EnvironmentSchema::default();
        assert_eq!(schema.name, "default");
        assert_eq!(schema.version, "1.0.0");
    }

    #[test]
    fn test_built_in_validator() {
        let validator = BuiltInValidator::NonEmpty;

        let mut update = StateUpdate::new();
        update.set("key", serde_json::json!("value"));

        let old_state = EnvironmentState::default();

        assert!(validator.validate(&old_state, &update).is_ok());
    }

    #[test]
    fn test_renderer_script() {
        let renderer = RendererScript {
            language: "rhai".to_string(),
            script: "let data = state.data; `{{data}}`".to_string(),
        };

        let state = EnvironmentState::default();

        // Rhai脚本会失败因为state变量不存在，但这是预期的
        let result = renderer.render(&state);
        assert!(result.is_err() || result.is_ok());
    }
}
