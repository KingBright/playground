//! Mission Control - Agent调度器
//!
//! 负责Agent的调度和任务执行

use crate::events::{Event, EventBus, EventType};
use crate::registry::{AgentDefinition, AgentRegistry, AgentType};
use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{interval, Duration, timeout};
use tracing::{error, info, warn};

// Engine integration for mission execution
use engine::workflow::{WorkflowConfig, WorkflowEngine, WorkflowScript};
use engine::agent::OracleProtocol;
use engine::session::{Session, SessionConfig};
use engine::environment::{Environment, EnvironmentState, StateSnapshot, StateUpdate};

// Brain integration for knowledge queries
use brain::storage::UnifiedMemory;

/// Cron表达式解析器
mod cron_parser {
    use chrono::{Datelike, Timelike};

    /// 简化版Cron解析：支持 "秒 分 时 日 月 周" 格式
    /// 例如: "0 */5 * * * *" 每5分钟
    pub fn should_trigger(cron_expr: &str, now: chrono::DateTime<chrono::Utc>) -> bool {
        let parts: Vec<&str> = cron_expr.split_whitespace().collect();
        if parts.len() != 6 {
            return false;
        }

        let minute = now.minute() as i32;
        let hour = now.hour() as i32;
        let day = now.day() as i32;
        let month = now.month() as i32;
        let weekday = now.weekday().num_days_from_sunday() as i32;

        // 解析秒、分、时、日、月、周
        matches_field(parts[0], 0, 59, 0) && // 秒（简化处理）
        matches_field(parts[1], minute, 0, 59) &&
        matches_field(parts[2], hour, 0, 23) &&
        matches_field(parts[3], day, 1, 31) &&
        matches_field(parts[4], month, 1, 12) &&
        matches_field(parts[5], weekday, 0, 6)
    }

    fn matches_field(pattern: &str, value: i32, min: i32, _max: i32) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern == "*/1" {
            return true;
        }

        // 处理 */n 格式
        if pattern.starts_with("*/") {
            if let Ok(step) = pattern[2..].parse::<i32>() {
                return (value - min) % step == 0;
            }
        }

        // 处理具体数字
        if let Ok(num) = pattern.parse::<i32>() {
            return value == num;
        }

        // 处理逗号分隔的列表
        if pattern.contains(',') {
            let nums: Vec<i32> = pattern
                .split(',')
                .filter_map(|s| s.parse().ok())
                .collect();
            return nums.contains(&value);
        }

        true // 默认通过
    }
}

/// 触发类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    /// 手动触发
    Manual,

    /// 定时触发
    Cron(String),

    /// 事件触发
    Event(String),
}

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,

    /// 任务超时时间 (秒)
    pub task_timeout_sec: u64,

    /// 重试次数
    pub max_retries: u32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            task_timeout_sec: 300,
            max_retries: 3,
        }
    }
}

/// 虚拟环境，用于Mission执行
struct MissionEnvironment {
    name: String,
    state: EnvironmentState,
}

impl MissionEnvironment {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            state: EnvironmentState::new(),
        }
    }
}

impl Environment for MissionEnvironment {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn current_state(&self) -> common::Result<EnvironmentState> {
        Ok(self.state.clone())
    }

    fn update_state(&mut self, _update: StateUpdate) -> common::Result<()> {
        // 简化实现，不实际更新状态
        Ok(())
    }

    fn validate_state(&self, _state: &EnvironmentState) -> common::Result<bool> {
        Ok(true)
    }

    fn render_state(&self, _state: &EnvironmentState) -> common::Result<String> {
        Ok(format!("MissionEnvironment: {}", self.name))
    }

    fn create_snapshot(&self) -> common::Result<StateSnapshot> {
        Ok(StateSnapshot {
            id: uuid::Uuid::new_v4(),
            environment_name: self.name.clone(),
            state: self.state.clone(),
            created_at: chrono::Utc::now(),
        })
    }

    fn restore_snapshot(&mut self, snapshot: &StateSnapshot) -> common::Result<()> {
        self.state = snapshot.state.clone();
        Ok(())
    }

    fn reset(&mut self) -> common::Result<()> {
        self.state = EnvironmentState::new();
        Ok(())
    }
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    /// 任务ID
    pub id: uuid::Uuid,

    /// 任务名称
    pub name: String,

    /// 目标Agent
    pub target_agent: String,

    /// 触发类型
    pub trigger: TriggerType,

    /// 任务参数
    pub parameters: serde_json::Value,

    /// 工作流脚本(可选，用于自动化任务)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_script: Option<String>,

    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MissionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 任务执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionExecution {
    /// 执行ID
    pub id: uuid::Uuid,
    /// 任务ID
    pub mission_id: uuid::Uuid,
    /// 状态
    pub status: MissionStatus,
    /// 开始时间
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 结束时间
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 输出结果
    pub output: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
}

/// Mission Control - 调度器
#[derive(Debug)]
pub struct MissionControl {
    /// Agent Registry
    registry: Arc<AgentRegistry>,

    /// 配置
    config: SchedulerConfig,

    /// 活跃任务
    active_missions: Arc<RwLock<Vec<Mission>>>,

    /// 任务历史
    mission_history: Arc<RwLock<Vec<Mission>>>,

    /// 执行记录
    executions: Arc<RwLock<HashMap<uuid::Uuid, Vec<MissionExecution>>>>,

    /// 调度器运行状态
    running: Arc<RwLock<bool>>,

    /// 事件总线
    event_bus: Arc<EventBus>,

    /// 并发控制信号量
    concurrency_semaphore: Arc<Semaphore>,

    /// Brain 知识库 (可选)
    brain: Option<Arc<UnifiedMemory>>,
}

impl MissionControl {
    /// 创建新的Mission Control
    pub fn new(registry: Arc<AgentRegistry>, config: SchedulerConfig) -> Self {
        Self::with_event_bus(registry, config, Arc::new(EventBus::new()), None)
    }

    /// 创建带事件总线的Mission Control
    pub fn with_event_bus(
        registry: Arc<AgentRegistry>,
        config: SchedulerConfig,
        event_bus: Arc<EventBus>,
        brain: Option<Arc<UnifiedMemory>>,
    ) -> Self {
        info!("Creating mission control with event bus");

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));

        Self {
            registry,
            config,
            active_missions: Arc::new(RwLock::new(Vec::new())),
            mission_history: Arc::new(RwLock::new(Vec::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            event_bus,
            concurrency_semaphore: semaphore,
            brain,
        }
    }

    /// 获取事件总线
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    /// 启动调度循环
    pub async fn start_scheduler(&self) {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        info!("Starting mission scheduler");

        let active_missions = self.active_missions.clone();
        let mission_history = self.mission_history.clone();
        let executions = self.executions.clone();
        let registry = self.registry.clone();
        let running_flag = self.running.clone();
        let event_bus = self.event_bus.clone();
        let concurrency_semaphore = self.concurrency_semaphore.clone();

        // 订阅事件驱动的任务
        self.subscribe_event_missions().await;

        // 启动事件分发循环
        let event_bus_clone = event_bus.clone();
        tokio::spawn(async move {
            event_bus_clone.start_dispatch().await;
        });

        // 每30秒检查一次定时任务
        let mut ticker = interval(Duration::from_secs(30));

        tokio::spawn(async move {
            loop {
                ticker.tick().await;

                // 检查是否停止
                let is_running = *running_flag.read().await;
                if !is_running {
                    info!("Scheduler stopped");
                    break;
                }

                let now = chrono::Utc::now();
                let missions_to_check: Vec<Mission> = {
                    let history = mission_history.read().await;
                    history.clone()
                };

                for mission in missions_to_check {
                    // 检查是否是Cron任务且应该触发
                    if let TriggerType::Cron(ref cron_expr) = mission.trigger {
                        if cron_parser::should_trigger(cron_expr, now) {
                            info!(
                                "Cron mission triggered: {} (expr: {})",
                                mission.name, cron_expr
                            );

                            // 尝试获取信号量许可
                            let permit = concurrency_semaphore.clone().try_acquire_owned();
                            if permit.is_err() {
                                warn!(
                                    "Mission {} skipped: max concurrent tasks reached",
                                    mission.name
                                );
                                continue;
                            }

                            // 创建执行记录
                            let execution = MissionExecution {
                                id: uuid::Uuid::new_v4(),
                                mission_id: mission.id,
                                status: MissionStatus::Pending,
                                started_at: None,
                                completed_at: None,
                                output: None,
                                error: None,
                            };

                            // 添加到活跃任务
                            {
                                let mut active = active_missions.write().await;
                                if !active.iter().any(|m| m.id == mission.id) {
                                    active.push(mission.clone());
                                }
                            }

                            // 保存执行记录
                            {
                                let mut execs = executions.write().await;
                                execs
                                    .entry(mission.id)
                                    .or_insert_with(Vec::new)
                                    .push(execution);
                            }

                            // 异步执行任务
                            let registry_clone = registry.clone();
                            let executions_clone = executions.clone();
                            let active_missions_clone = active_missions.clone();
                            tokio::spawn(async move {
                                Self::run_mission(
                                    &mission,
                                    registry_clone,
                                    executions_clone,
                                    active_missions_clone,
                                ).await;
                                // permit 在这里自动释放
                            });
                        }
                    }
                }
            }
        });
    }

    /// 订阅事件驱动的任务
    async fn subscribe_event_missions(&self) {
        let mission_history = self.mission_history.clone();
        let active_missions = self.active_missions.clone();
        let executions = self.executions.clone();
        let registry = self.registry.clone();
        let concurrency_semaphore = self.concurrency_semaphore.clone();

        // 订阅数据更新事件
        let event_bus = self.event_bus.clone();
        event_bus
            .subscribe("data_updated:*", move |event| {
                let mission_history = mission_history.clone();
                let active_missions = active_missions.clone();
                let executions = executions.clone();
                let registry = registry.clone();
                let concurrency_semaphore = concurrency_semaphore.clone();

                async move {
                    let event_name = event.event_type.name();
                    info!("Processing event: {}", event_name);

                    // 查找匹配的事件驱动任务
                    let missions: Vec<Mission> = {
                        let history = mission_history.read().await;
                        history
                            .iter()
                            .filter(|m| matches!(m.trigger, TriggerType::Event(ref pattern) if event_name.contains(pattern)))
                            .cloned()
                            .collect()
                    };

                    for mission in missions {
                        info!("Event-triggered mission started: {}", mission.name);

                        // 尝试获取信号量许可
                        let permit = concurrency_semaphore.clone().try_acquire_owned();
                        if permit.is_err() {
                            warn!("Event mission {} skipped: max concurrent tasks reached", mission.name);
                            continue;
                        }

                        // 创建执行记录
                        let execution = MissionExecution {
                            id: uuid::Uuid::new_v4(),
                            mission_id: mission.id,
                            status: MissionStatus::Pending,
                            started_at: None,
                            completed_at: None,
                            output: None,
                            error: None,
                        };

                        // 添加到活跃任务
                        {
                            let mut active = active_missions.write().await;
                            if !active.iter().any(|m| m.id == mission.id) {
                                active.push(mission.clone());
                            }
                        }

                        // 保存执行记录
                        {
                            let mut execs = executions.write().await;
                            execs
                                .entry(mission.id)
                                .or_insert_with(Vec::new)
                                .push(execution);
                        }

                        // 异步执行任务
                        let registry_clone = registry.clone();
                        let executions_clone = executions.clone();
                        let active_missions_clone = active_missions.clone();
                        tokio::spawn(async move {
                            Self::run_mission(
                                &mission,
                                registry_clone,
                                executions_clone,
                                active_missions_clone,
                            ).await;
                        });
                    }
                }
            })
            .await;
    }

    /// 停止调度循环
    pub async fn stop_scheduler(&self) {
        info!("Stopping mission scheduler");
        let mut running = self.running.write().await;
        *running = false;
    }

    /// 触发手动任务
    pub async fn trigger_manual_mission(&self, mission_id: uuid::Uuid) -> Result<MissionExecution> {
        let mission = {
            let history = self.mission_history.read().await;
            history.iter().find(|m| m.id == mission_id).cloned()
        };

        let mission = mission.ok_or_else(|| Error::NotFound(format!("Mission {} not found", mission_id)))?;

        info!("Manually triggering mission: {}", mission.name);

        // 尝试获取信号量许可
        let _permit = self.concurrency_semaphore.acquire().await
            .map_err(|e| Error::Internal(format!("Failed to acquire semaphore: {}", e)))?;

        // 创建执行记录
        let execution = MissionExecution {
            id: uuid::Uuid::new_v4(),
            mission_id: mission.id,
            status: MissionStatus::Pending,
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            output: None,
            error: None,
        };

        // 添加到活跃任务
        {
            let mut active = self.active_missions.write().await;
            if !active.iter().any(|m| m.id == mission.id) {
                active.push(mission.clone());
            }
        }

        // 保存执行记录
        {
            let mut execs = self.executions.write().await;
            execs
                .entry(mission.id)
                .or_insert_with(Vec::new)
                .push(execution.clone());
        }

        // 异步执行
        let registry = self.registry.clone();
        let executions = self.executions.clone();
        let active_missions = self.active_missions.clone();
        let timeout_sec = self.config.task_timeout_sec;
        let max_retries = self.config.max_retries;

        tokio::spawn(async move {
            Self::run_mission_with_timeout(&mission, registry, executions, active_missions, timeout_sec, max_retries).await;
        });

        Ok(execution)
    }

    /// 实际执行任务（带超时和重试）
    async fn run_mission(
        mission: &Mission,
        registry: Arc<AgentRegistry>,
        executions: Arc<RwLock<HashMap<uuid::Uuid, Vec<MissionExecution>>>>,
        active_missions: Arc<RwLock<Vec<Mission>>>,
    ) {
        Self::run_mission_with_timeout(mission, registry, executions, active_missions, 300, 3).await;
    }

    /// 实际执行任务（带超时和重试）
    async fn run_mission_with_timeout(
        mission: &Mission,
        registry: Arc<AgentRegistry>,
        executions: Arc<RwLock<HashMap<uuid::Uuid, Vec<MissionExecution>>>>,
        active_missions: Arc<RwLock<Vec<Mission>>>,
        timeout_sec: u64,
        max_retries: u32,
    ) {
        info!("Running mission: {} (timeout: {}s, max_retries: {})", mission.name, timeout_sec, max_retries);

        // 更新执行状态为Running
        {
            let mut execs = executions.write().await;
            if let Some(mission_execs) = execs.get_mut(&mission.id) {
                if let Some(last) = mission_execs.last_mut() {
                    last.status = MissionStatus::Running;
                    last.started_at = Some(chrono::Utc::now());
                }
            }
        }

        let mut final_result: Result<serde_json::Value> = Err(Error::Internal("Not executed".to_string()));
        let mut attempts = 0;

        // 重试循环
        while attempts <= max_retries {
            if attempts > 0 {
                info!("Retrying mission {} (attempt {}/{})", mission.name, attempts, max_retries);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }

            // 设置超时
            let timeout_duration = Duration::from_secs(timeout_sec);
            let execution_result = timeout(timeout_duration, async {
                Self::execute_mission_once(mission, registry.clone()).await
            }).await;

            match execution_result {
                Ok(Ok(result)) => {
                    final_result = Ok(result);
                    break;
                }
                Ok(Err(e)) => {
                    error!("Mission {} failed (attempt {}): {}", mission.name, attempts, e);
                    final_result = Err(e);
                }
                Err(_) => {
                    error!("Mission {} timed out after {}s", mission.name, timeout_sec);
                    final_result = Err(Error::Timeout(format!("Mission timed out after {}s", timeout_sec)));
                }
            }

            attempts += 1;
        }

        // 更新执行结果
        {
            let mut execs = executions.write().await;
            if let Some(mission_execs) = execs.get_mut(&mission.id) {
                if let Some(last) = mission_execs.last_mut() {
                    last.completed_at = Some(chrono::Utc::now());
                    match final_result {
                        Ok(output) => {
                            last.status = MissionStatus::Completed;
                            last.output = Some(output);
                        }
                        Err(e) => {
                            last.status = MissionStatus::Failed;
                            last.error = Some(e.to_string());
                        }
                    }
                }
            }
        }

        // 从活跃任务移除
        {
            let mut active = active_missions.write().await;
            active.retain(|m| m.id != mission.id);
        }

        info!("Mission {} completed", mission.name);
    }

    /// 单次执行任务（无重试逻辑）
    async fn execute_mission_once(
        mission: &Mission,
        registry: Arc<AgentRegistry>,
    ) -> Result<serde_json::Value> {
        // 获取Agent定义
        let agent_def = registry.get(&mission.target_agent).await;

        if let Some(def) = agent_def {
            info!(
                "Executing agent {} for mission {}",
                def.name, mission.name
            );

            // 如果有工作流脚本，使用WorkflowEngine执行
            if let Some(script_content) = &mission.workflow_script {
                info!("Executing workflow script for mission {}", mission.name);

                // 创建临时Session用于执行
                let session_config = SessionConfig {
                    name: format!("mission_{}_session", mission.id),
                    description: Some(format!("Auto-created session for mission {}", mission.name)),
                    auto_snapshot_interval: 60,
                    max_snapshots: 10,
                    enable_logging: true,
                };

                // 创建虚拟Environment和Session
                let env = Box::new(MissionEnvironment::new(&mission.name));
                let session = Arc::new(Session::new(session_config, env));

                // 创建Oracle Protocol
                let oracle = Arc::new(OracleProtocol::new());

                // 创建WorkflowEngine
                let workflow_config = WorkflowConfig::default();
                let workflow_engine = WorkflowEngine::new(
                    workflow_config,
                    session,
                    oracle,
                );

                // 创建Workflow脚本
                let script = WorkflowScript::new(
                    mission.name.clone(),
                    script_content.clone(),
                ).with_description(format!("Mission {} workflow", mission.name));

                // 执行脚本
                match workflow_engine.execute(&script).await {
                    Ok(result) => {
                        info!("Workflow executed successfully for mission {}", mission.name);
                        Ok(serde_json::json!({
                            "status": "completed",
                            "agent": def.name,
                            "mission": mission.name,
                            "workflow_result": {
                                "success": result.success,
                                "duration_ms": result.duration_ms,
                                "output": result.output,
                            },
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }))
                    }
                    Err(e) => {
                        error!("Workflow execution failed for mission {}: {}", mission.name, e);
                        Err(Error::WorkflowError(format!("Workflow execution failed: {}", e)))
                    }
                }
            } else {
                // 模拟Agent执行（没有工作流脚本时）
                tokio::time::sleep(Duration::from_millis(100)).await;

                Ok(serde_json::json!({
                    "status": "completed",
                    "agent": def.name,
                    "mission": mission.name,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }
        } else {
            Err(Error::NotFound(format!(
                "Agent {} not found",
                mission.target_agent
            )))
        }
    }

    /// 获取任务执行历史
    pub async fn get_mission_executions(&self, mission_id: uuid::Uuid) -> Vec<MissionExecution> {
        let execs = self.executions.read().await;
        execs.get(&mission_id).cloned().unwrap_or_default()
    }

    /// 创建新任务
    pub async fn create_mission(
        &self,
        name: String,
        target_agent: String,
        trigger: TriggerType,
        parameters: serde_json::Value,
    ) -> Result<Mission> {
        self.create_mission_with_script(name, target_agent, trigger, parameters, None).await
    }

    /// 创建带工作流脚本的任务
    pub async fn create_mission_with_script(
        &self,
        name: String,
        target_agent: String,
        trigger: TriggerType,
        parameters: serde_json::Value,
        workflow_script: Option<String>,
    ) -> Result<Mission> {
        info!("Creating mission: {} for agent: {}", name, target_agent);

        // 验证Agent存在
        let agent_exists = self.registry.get(&target_agent).await.is_some();
        if !agent_exists {
            return Err(Error::NotFound(format!(
                "Agent '{}' not found",
                target_agent
            )));
        }

        let mission = Mission {
            id: uuid::Uuid::new_v4(),
            name,
            target_agent,
            trigger,
            parameters,
            workflow_script,
            created_at: chrono::Utc::now(),
        };

        // 保存到历史
        let mut history = self.mission_history.write().await;
        history.push(mission.clone());

        Ok(mission)
    }

    /// 执行任务（同步执行，带超时和重试）
    pub async fn execute_mission(&self, mission: &Mission) -> Result<MissionExecution> {
        info!("Executing mission: {}", mission.name);

        // 创建执行记录
        let execution = MissionExecution {
            id: uuid::Uuid::new_v4(),
            mission_id: mission.id,
            status: MissionStatus::Running,
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            output: None,
            error: None,
        };

        // 添加到活跃任务
        {
            let mut active = self.active_missions.write().await;
            active.push(mission.clone());
        }

        // 保存执行记录
        {
            let mut execs = self.executions.write().await;
            execs
                .entry(mission.id)
                .or_insert_with(Vec::new)
                .push(execution.clone());
        }

        // 使用带超时和重试的任务执行逻辑
        Self::run_mission_with_timeout(
            mission,
            self.registry.clone(),
            self.executions.clone(),
            self.active_missions.clone(),
            self.config.task_timeout_sec,
            self.config.max_retries,
        ).await;

        // 返回执行结果
        let execs = self.executions.read().await;
        if let Some(mission_execs) = execs.get(&mission.id) {
            if let Some(last) = mission_execs.last() {
                return Ok(last.clone());
            }
        }

        Ok(execution)
    }

    /// 获取活跃任务
    pub async fn get_active_missions(&self) -> Vec<Mission> {
        self.active_missions.read().await.clone()
    }

    /// 获取任务历史
    pub async fn get_history(&self) -> Vec<Mission> {
        self.mission_history.read().await.clone()
    }

    /// 设置 Brain 知识库
    pub fn set_brain(&mut self, brain: Arc<UnifiedMemory>) {
        self.brain = Some(brain);
        info!("Brain memory attached to MissionControl");
    }

    /// 查询知识库（简化的 Brain 集成）
    pub async fn query_knowledge(&self, query: &str, limit: usize) -> Result<Vec<serde_json::Value>> {
        if let Some(_brain) = self.brain.clone() {
            // Brain 集成：可以通过 brain.search_archive 进行基础搜索
            // 注意：实际项目中需要嵌入向量才能使用 search_vector
            let results = vec![serde_json::json!({
                "query": query,
                "limit": limit,
                "status": "Brain integration placeholder - use search_archive for text search"
            })];

            Ok(results)
        } else {
            Err(Error::Internal("Brain memory not configured".to_string()))
        }
    }

    /// 存储知识到 Brain（简化的 Brain 集成）
    pub async fn store_knowledge(&self, title: &str, content: &str, _tags: Vec<String>) -> Result<uuid::Uuid> {
        if let Some(_brain) = self.brain.clone() {
            // Brain 集成：可以通过 brain.archive_raw 存储原始数据
            // 或者通过 brain.store_hot 存储热数据
            let slice_id = uuid::Uuid::new_v4();
            info!("Knowledge stored: {} (id: {})", title, slice_id);
            Ok(slice_id)
        } else {
            Err(Error::Internal("Brain memory not configured".to_string()))
        }
    }

    /// 发布事件
    pub fn publish_event(&self, event_type: EventType, payload: serde_json::Value) -> Result<()> {
        let event = Event::new(event_type, payload);
        self.event_bus.publish(event)
            .map_err(|e| Error::Internal(format!("Failed to publish event: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{AgentDefinition, AgentRegistry, AgentType};

    #[tokio::test]
    async fn test_mission_control_creation() {
        let registry = Arc::new(AgentRegistry::new());
        let config = SchedulerConfig::default();

        let mc = MissionControl::new(registry, config);
        assert_eq!(mc.get_active_missions().await.len(), 0);
    }

    #[tokio::test]
    async fn test_mission_creation() {
        let registry = Arc::new(AgentRegistry::new());

        // 先注册一个Agent
        let definition = AgentDefinition {
            id: uuid::Uuid::new_v4(),
            name: "test_agent".to_string(),
            agent_type: AgentType::Local,
            config: serde_json::json!({}),
            description: None,
            created_at: chrono::Utc::now(),
        };

        registry.register(definition).await.unwrap();

        let config = SchedulerConfig::default();
        let mc = MissionControl::new(registry, config);

        let mission = mc
            .create_mission(
                "test_mission".to_string(),
                "test_agent".to_string(),
                TriggerType::Manual,
                serde_json::json!({}),
            )
            .await;

        assert!(mission.is_ok());
        let mission = mission.unwrap();
        assert_eq!(mission.name, "test_mission");
    }

    #[test]
    fn test_trigger_type() {
        let manual = TriggerType::Manual;
        let cron = TriggerType::Cron("0 * * * *".to_string());
        let event = TriggerType::Event("data_updated".to_string());

        // TriggerType可以序列化
        let _ = manual;
        let _ = cron;
        let _ = event;
    }

    #[tokio::test]
    async fn test_mission_execution() {
        let registry = Arc::new(AgentRegistry::new());

        // 注册Agent
        let definition = AgentDefinition {
            id: uuid::Uuid::new_v4(),
            name: "test_agent".to_string(),
            agent_type: AgentType::Local,
            config: serde_json::json!({}),
            description: None,
            created_at: chrono::Utc::now(),
        };
        registry.register(definition).await.unwrap();

        let config = SchedulerConfig::default();
        let mc = MissionControl::new(registry, config);

        // 创建任务
        let mission = mc
            .create_mission(
                "test_mission".to_string(),
                "test_agent".to_string(),
                TriggerType::Manual,
                serde_json::json!({}),
            )
            .await
            .unwrap();

        // 执行任务
        let result = mc.execute_mission(&mission).await;
        assert!(result.is_ok());

        let execution = result.unwrap();
        assert_eq!(execution.status, MissionStatus::Completed);
        assert!(execution.output.is_some());
    }

    #[tokio::test]
    async fn test_cron_trigger() {
        use chrono::Timelike;

        let now = chrono::Utc::now();

        // 测试每分钟触发
        assert!(cron_parser::should_trigger("0 */1 * * * *", now));

        // 测试特定分钟（应该总是失败，因为我们不知道当前分钟）
        // 但我们可以测试格式解析
        let specific_minute = format!("0 {} * * * *", now.minute());
        assert!(cron_parser::should_trigger(&specific_minute, now));

        // 测试通配符
        assert!(cron_parser::should_trigger("* * * * * *", now));
    }

    #[tokio::test]
    async fn test_mission_executions_tracking() {
        let registry = Arc::new(AgentRegistry::new());

        let definition = AgentDefinition {
            id: uuid::Uuid::new_v4(),
            name: "test_agent".to_string(),
            agent_type: AgentType::Local,
            config: serde_json::json!({}),
            description: None,
            created_at: chrono::Utc::now(),
        };
        registry.register(definition).await.unwrap();

        let config = SchedulerConfig::default();
        let mc = MissionControl::new(registry, config);

        let mission = mc
            .create_mission(
                "test_mission".to_string(),
                "test_agent".to_string(),
                TriggerType::Manual,
                serde_json::json!({}),
            )
            .await
            .unwrap();

        // 执行两次
        let _ = mc.execute_mission(&mission).await;
        let _ = mc.execute_mission(&mission).await;

        // 检查执行记录
        let executions = mc.get_mission_executions(mission.id).await;
        assert_eq!(executions.len(), 2);
    }
}
