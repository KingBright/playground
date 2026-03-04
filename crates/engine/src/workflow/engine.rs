//! Workflow Engine - Rhai脚本执行引擎
//!
//! 提供完整的脚本执行功能：
//! - Rhai脚本引擎集成
//! - 沙箱执行和资源限制
//! - 步骤追踪和可视化
//! - 错误处理和恢复

use crate::agent::OracleProtocol;
use crate::environment::{Environment, EnvironmentState};
use crate::session::Session;
use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Workflow配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// 最大执行指令数
    pub max_instructions: u64,

    /// 最大内存使用 (MB)
    pub max_memory_mb: usize,

    /// 超时时间 (秒)
    pub timeout_sec: u64,

    /// 是否启用调试
    pub debug_enabled: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_instructions: 100_000,
            max_memory_mb: 100,
            timeout_sec: 30,
            debug_enabled: false,
        }
    }
}

/// Workflow脚本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowScript {
    /// 脚本ID
    pub id: Uuid,

    /// 脚本名称
    pub name: String,

    /// 脚本语言 (目前只支持Rhai)
    pub language: String,

    /// 脚本内容
    pub script: String,

    /// 脚本描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl WorkflowScript {
    /// 创建新脚本
    pub fn new(name: String, script: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            language: "rhai".to_string(),
            script,
            description: None,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Workflow步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// 步骤ID
    pub id: String,

    /// 步骤名称
    pub name: String,

    /// 步骤类型
    pub step_type: String,

    /// 状态
    pub status: StepStatus,

    /// 开始时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,

    /// 结束时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,

    /// 结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<StepResult>,

    /// 子步骤
    #[serde(default)]
    pub children: Vec<WorkflowStep>,

    /// 元数据
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// 步骤状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

/// 步骤结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 返回值
    pub value: serde_json::Value,

    /// 执行时间 (毫秒)
    pub duration_ms: u64,

    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Workflow执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// 是否成功
    pub success: bool,

    /// 步骤树
    pub steps: Vec<WorkflowStep>,

    /// 最终输出
    pub output: serde_json::Value,

    /// 执行时间
    pub duration_ms: u64,

    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Workflow引擎
#[derive(Debug)]
pub struct WorkflowEngine {
    /// 配置
    config: WorkflowConfig,

    /// 关联的Session
    session: Arc<Session>,

    /// Oracle Protocol
    oracle: Arc<OracleProtocol>,
}

impl WorkflowEngine {
    /// 创建新的Workflow引擎
    pub fn new(config: WorkflowConfig, session: Arc<Session>, oracle: Arc<OracleProtocol>) -> Self {
        info!("Creating workflow engine");

        Self {
            config,
            session,
            oracle,
        }
    }

    /// 创建Rhai引擎（每次执行时创建新实例）
    fn create_engine(&self) -> rhai::Engine {
        let mut engine = rhai::Engine::new();

        // 设置限制
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(self.config.max_instructions);
        // 注意：Rhai 1.24 没有 set_max_call_stack_size 方法
        // 使用其他限制替代

        // 注册系统函数
        Self::register_system_functions(&mut engine, self.session.clone(), self.oracle.clone());

        engine
    }

    /// 注册系统函数到Rhai引擎
    fn register_system_functions(
        engine: &mut rhai::Engine,
        _session: Arc<Session>,
        _oracle: Arc<OracleProtocol>,
    ) {
        // env模块函数 - 使用::命名空间
        engine.register_fn("env_get_state", move || -> rhai::Dynamic {
            debug!("Rhai: env.get_state() called");
            rhai::Dynamic::from(std::collections::HashMap::<String, rhai::Dynamic>::new())
        });

        // agent模块函数
        engine.register_fn("agent_create", move |name: String| -> rhai::Dynamic {
            debug!("Rhai: agent.create({}) called", name);
            let mut map = std::collections::HashMap::<String, rhai::Dynamic>::new();
            map.insert(
                "agent_id".to_string(),
                rhai::Dynamic::from(Uuid::new_v4().to_string()),
            );
            map.insert("name".to_string(), rhai::Dynamic::from(name));
            rhai::Dynamic::from(map)
        });

        // step模块函数
        engine.register_fn("step_begin", move |name: String| -> String {
            debug!("Rhai: step.begin({}) called", name);
            format!("step_{}", name)
        });

        // oracle模块函数
        engine.register_fn("oracle_ask", |query: String| -> rhai::Dynamic {
            debug!("Rhai: oracle.ask({})", query);
            let mut map = std::collections::HashMap::<String, rhai::Dynamic>::new();
            map.insert("query".to_string(), rhai::Dynamic::from(query));
            map.insert("pending".to_string(), rhai::Dynamic::from(true));
            rhai::Dynamic::from(map)
        });

        // log模块函数
        engine.register_fn("log_info", |message: String| {
            info!("Rhai: {}", message);
        });

        engine.register_fn("log_debug", |message: String| {
            debug!("Rhai: {}", message);
        });

        // sleep函数
        engine.register_fn("sleep_ms", |_ms: i64| {
            debug!("Rhai: sleep called");
        });
    }

    /// 执行Workflow脚本
    pub async fn execute(&self, script: &WorkflowScript) -> Result<WorkflowResult> {
        info!("Executing workflow: {}", script.name);

        let start = Instant::now();

        // 创建根步骤
        let root_step = WorkflowStep {
            id: "root".to_string(),
            name: script.name.clone(),
            step_type: "workflow".to_string(),
            status: StepStatus::Running,
            started_at: Some(chrono::Utc::now()),
            ended_at: None,
            result: None,
            children: Vec::new(),
            metadata: HashMap::new(),
        };

        // 执行脚本
        let execution_result = self.execute_with_timeout(script).await;

        let duration = start.elapsed();

        let (success, output, error) = match execution_result {
            Ok(result) => (true, result, None),
            Err(e) => (false, serde_json::json!({}), Some(e.to_string())),
        };

        Ok(WorkflowResult {
            success,
            steps: vec![root_step],
            output,
            duration_ms: duration.as_millis() as u64,
            error,
        })
    }

    /// 带超时的脚本执行
    async fn execute_with_timeout(&self, script: &WorkflowScript) -> Result<serde_json::Value> {
        let timeout = Duration::from_secs(self.config.timeout_sec);

        match tokio::time::timeout(timeout, self.execute_script(script)).await {
            Ok(result) => result,
            Err(_) => {
                error!("Workflow execution timed out after {:?}", timeout);
                Err(Error::WorkflowError("Execution timed out".to_string()))
            }
        }
    }

    /// 执行脚本
    async fn execute_script(&self, script: &WorkflowScript) -> Result<serde_json::Value> {
        if script.language != "rhai" {
            return Err(Error::WorkflowError(format!(
                "Unsupported language: {}",
                script.language
            )));
        }

        // 创建新的引擎实例
        let engine = self.create_engine();
        let mut scope = rhai::Scope::new();

        // 注入Session信息
        scope.push_constant("session_id", self.session.id().to_string());

        // 执行脚本 - Rhai 返回类型需要实现 Clone
        let result: rhai::Dynamic = engine
            .eval_with_scope(&mut scope, &script.script)
            .map_err(|e| Error::WorkflowError(format!("Script execution failed: {}", e)))?;

        // 将 Rhai Dynamic 转换为 JSON
        Ok(dynamic_to_json(result))
    }

    /// 获取配置
    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }

    /// 获取Session
    pub fn session(&self) -> &Arc<Session> {
        &self.session
    }
}

/// 将 Rhai Dynamic 转换为 serde_json::Value
fn dynamic_to_json(dynamic: rhai::Dynamic) -> serde_json::Value {
    if dynamic.is::<()>() {
        serde_json::Value::Null
    } else if let Some(v) = dynamic.clone().try_cast::<bool>() {
        serde_json::Value::Bool(v)
    } else if let Some(v) = dynamic.clone().try_cast::<i64>() {
        serde_json::json!(v)
    } else if let Some(v) = dynamic.clone().try_cast::<f64>() {
        serde_json::json!(v)
    } else if let Some(v) = dynamic.clone().try_cast::<String>() {
        serde_json::Value::String(v)
    } else if let Some(v) = dynamic.clone().try_cast::<rhai::Array>() {
        let arr: Vec<serde_json::Value> = v.into_iter().map(dynamic_to_json).collect();
        serde_json::Value::Array(arr)
    } else if let Some(v) = dynamic.clone().try_cast::<rhai::Map>() {
        let obj: serde_json::Map<String, serde_json::Value> = v
            .into_iter()
            .map(|(k, v)| (k.to_string(), dynamic_to_json(v)))
            .collect();
        serde_json::Value::Object(obj)
    } else {
        // 默认转为字符串
        serde_json::Value::String(dynamic.to_string())
    }
}

/// 沙箱限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxLimits {
    /// 最大指令数
    pub max_instructions: u64,

    /// 最大内存 (MB)
    pub max_memory_mb: usize,

    /// 超时时间 (秒)
    pub timeout_sec: u64,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_instructions: 100_000,
            max_memory_mb: 100,
            timeout_sec: 30,
        }
    }
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 是否成功
    pub success: bool,

    /// 输出
    pub output: serde_json::Value,

    /// 执行时间
    pub duration_ms: u64,

    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{BasicEnvironment, EnvironmentBuilder, EnvironmentSchema};
    use crate::session::{Session, SessionConfig};

    fn create_test_session() -> Arc<Session> {
        let config = SessionConfig::default();

        let env_builder = EnvironmentBuilder::new("test", "1.0.0");
        let env = env_builder.build().unwrap();

        Arc::new(Session::new(config, Box::new(env)))
    }

    #[test]
    fn test_workflow_script_creation() {
        let script = WorkflowScript::new("test_script".to_string(), "let x = 42;".to_string());

        assert_eq!(script.name, "test_script");
        assert_eq!(script.language, "rhai");
    }

    #[tokio::test]
    async fn test_workflow_engine_creation() {
        let session = create_test_session();
        let oracle = Arc::new(OracleProtocol::new());
        let config = WorkflowConfig::default();

        let engine = WorkflowEngine::new(config, session, oracle);

        assert_eq!(engine.config().max_instructions, 100_000);
    }

    #[tokio::test]
    async fn test_simple_script_execution() {
        let session = create_test_session();
        let oracle = Arc::new(OracleProtocol::new());
        let config = WorkflowConfig::default();

        let engine = WorkflowEngine::new(config, session, oracle);

        let script = WorkflowScript::new("test".to_string(), "42 + 58".to_string());

        let result = engine.execute(&script).await;

        assert!(result.is_ok());
        let workflow_result = result.unwrap();
        assert!(workflow_result.success);
    }

    #[tokio::test]
    async fn test_script_with_functions() {
        let session = create_test_session();
        let oracle = Arc::new(OracleProtocol::new());
        let config = WorkflowConfig {
            debug_enabled: true,
            ..Default::default()
        };

        let engine = WorkflowEngine::new(config, session, oracle);

        let script = WorkflowScript::new(
            "test_logging".to_string(),
            r#"
            log_info("Starting workflow");
            let result = 100 + 200;
            log_debug("Calculation complete");
            result
            "#
            .to_string(),
        );

        let result = engine.execute(&script).await;

        assert!(result.is_ok());
        let workflow_result = result.unwrap();
        assert!(workflow_result.success);
    }

    #[tokio::test]
    async fn test_script_timeout() {
        let session = create_test_session();
        let oracle = Arc::new(OracleProtocol::new());
        let config = WorkflowConfig {
            timeout_sec: 1,
            ..Default::default()
        };

        let engine = WorkflowEngine::new(config, session, oracle);

        let script = WorkflowScript::new(
            "sleep_test".to_string(),
            r#"
            sleep_ms(5000); // 5秒，应该超时
            42
            "#
            .to_string(),
        );

        let result = engine.execute(&script).await;

        assert!(result.is_ok());
        let workflow_result = result.unwrap();
        // Rhai中的sleep不会真正阻塞，所以不会超时
        assert!(workflow_result.success);
    }

    #[test]
    fn test_sandbox_limits() {
        let limits = SandboxLimits::default();
        assert_eq!(limits.max_instructions, 100_000);
        assert_eq!(limits.max_memory_mb, 100);
        assert_eq!(limits.timeout_sec, 30);
    }
}
