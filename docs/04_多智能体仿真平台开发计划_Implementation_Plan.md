# 多智能体仿真平台开发计划

## 项目概述

基于 docs/ 目录中的三份架构文档，本计划将实现一个由三大核心系统组成的多智能体仿真平台：

| 系统 | 职责 |
|------|------|
| **External Brain** | 知识供应链 - 采集、清洗、存储、索引知识 |
| **Agent Playground** | 仿真运行时 - 基于 ECS 模式的 Agent 模拟引擎 |
| **System Synergy** | 神经中枢 - Agent 调度、注册、协同机制 |

## 当前项目状态 (2026-02-13 更新)

- **已完成**: ✅
  - ✅ Phase 0-3: Brain 系统 (100% - 存储/采集/处理层)
  - ✅ Phase 4-5: Engine 系统 (90% - Workflow引擎可运行)
  - ✅ Phase 6: Synergy 系统 (70% - 基础调度功能)
  - ✅ Phase 7: API 集成 (100% - 20个端点，全部测试通过)
- **进行中**: ⚠️ Synergy调度器完善、WebSocket支持
- **依赖**: tokio, serde, axum, rhai, redis, sqlx 等已配置

---

## 实现阶段概览 (7 个阶段)

```
Phase 0: 基础设施 (2周) ✅ 已完成
    ├── 测试框架
    ├── 配置系统
    └── 所有 crate 依赖此

Phase 1: Brain 存储层 (3周) ✅ 已完成 ──┐
Phase 2: Brain 采集层 (2周) ✅ 已完成   │ 并行
Phase 3: Brain 处理层 (2周) ✅ 已完成   │
                                         │
Phase 4: Engine 核心 (3周) ✅ 已完成 ───┤ 并行
Phase 5: Engine 逻辑 (4周) ✅ 90% ──────┤
    └── Workflow引擎已可运行            │
                                         │
Phase 6: Synergy 协同 (2周) ✅ 70% ─────┤ 依赖 Brain + Engine
    └── 基础功能完成，需完善调度执行    │
                                         │
Phase 7: API 集成 (2周) ✅ 已完成 ──────┘ 依赖所有系统
    └── 20个端点，全部测试通过
```

**实际进展**: 核心功能已完成，系统可运行。当前主要工作：
1. 完善Synergy调度器的实际执行逻辑
2. 添加WebSocket实时推送
3. 外部存储服务集成 (Redis/Qdrant/Neo4j)

---

## 详细实现计划

### Phase 0: 基础设施 (第 1-2 周)

#### 0.1 测试框架
**文件**: `crates/common/src/testing/mod.rs`
- Mock Agent 实现
- 测试数据构建器
- In-memory 存储后端

#### 0.2 配置系统
**文件**: `crates/common/src/config/mod.rs`
- 统一配置管理
- 环境变量支持
- 分模块配置验证

#### 0.3 LLM 客户端抽象
**文件**: `crates/common/src/llm/mod.rs`
```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}
```

---

### Phase 1: Brain 存储层 (第 3-5 周)

#### 1.1 核心 Memory Trait
**文件**: `crates/brain/src/storage/memory.rs`
- 统一的存储接口抽象
- 支持多种后端实现

#### 1.2 Hot Memory (Redis)
**文件**: `crates/brain/src/storage/hot_memory.rs`
- Redis 连接池
- 24小时 TTL 管理
- 异常重试机制

#### 1.3 Vector Memory (占位)
**文件**: `crates/brain/src/storage/vector_memory.rs`
- 定义语义搜索接口
- 先用 in-memory 实现 (cosine similarity)
- 后续可切换到 Qdrant/Milvus

#### 1.4 Graph Memory (占位)
**文件**: `crates/brain/src/storage/graph_memory.rs`
- 实体关系图接口
- 使用 `petgraph` crate 实现 in-memory 版本
- 后续可切换到 Neo4j

#### 1.5 Raw Archive
**文件**: `crates/brain/src/storage/raw_archive.rs`
- 文件系统存储
- JSON 格式 + 元数据索引

#### 1.6 Unified Memory Facade
**文件**: `crates/brain/src/storage/unified_memory.rs`
```rust
pub struct UnifiedMemory {
    hot: Arc<dyn HotMemoryBackend>,
    vector: Arc<dyn VectorMemoryBackend>,
    graph: Arc<dyn GraphMemoryBackend>,
    raw: Arc<dyn RawArchiveBackend>,
}
```

**里程碑**: 可存储和检索数据，跨后端搜索

---

### Phase 2: Brain 数据采集 (第 6-7 周)

#### 2.1 Collector Trait
**文件**: `crates/brain/src/collectors/mod.rs`
```rust
#[async_trait]
pub trait Collector: Send + Sync {
    async fn collect(&self) -> Result<Vec<RawData>>;
    fn name(&self) -> &str;
    fn schedule(&self) -> Option<CronSchedule>;
}
```

#### 2.2 API Collector
**文件**: `crates/brain/src/collectors/api_collector.rs`
- HTTP 客户端 (使用 `reqwest` crate)
- 分页处理、速率限制

#### 2.3 RSS Collector
**文件**: `crates/brain/src/collectors/rss_collector.rs`
- RSS 解析 (使用 `feed-rs` crate)
- 增量更新、去重

#### 2.4 File Upload Handler
**文件**: `crates/brain/src/collectors/file_handler.rs`
- 文件上传验证
- 类型检查、大小限制

**里程碑**: 可从多种数据源采集数据

---

### Phase 3: Brain 处理层 (第 8-9 周)

#### 3.1 Universal Agents 实现

**文件**: `crates/brain/src/processors/cleaner.rs`
- Cleaner Agent: 正则清洗、去噪

**文件**: `crates/brain/src/processors/extractor.rs`
- Extractor Agent: NER 实体抽取、关系抽取

**文件**: `crates/brain/src/processors/summarizer.rs`
- Summarizer Agent: 长文本摘要

**文件**: `crates/brain/src/processors/tagger.rs`
- Tagger Agent: 分类、打标、生成 embedding

#### 3.2 处理流水线
**文件**: `crates/brain/src/processors/pipeline.rs`
```rust
pub struct ProcessingPipeline {
    steps: Vec<PipelineStep>,
}
// Cleaner -> Tagger -> Extractor 链式处理
```

**里程碑**: Brain 系统完整可运行，可处理 RSS 新闻并检索

---

### Phase 4: Engine 核心 (第 10-12 周)

#### 4.1 Environment Schema 系统
**文件**: `crates/engine/src/environment/schema.rs`
```rust
pub struct EnvironmentSchema {
    pub state_definition: StateSchema,
    pub validators: Vec<ValidatorDef>,
    pub renderer: RendererScript,
}
```

#### 4.2 State Manager
**文件**: `crates/engine/src/environment/state.rs`
- 状态更新与验证
- Copy-on-Write 快照

#### 4.3 Session 生命周期
**文件**: `crates/engine/src/session/mod.rs`
```rust
pub struct Session {
    pub id: Uuid,
    pub environment: Environment,
    pub agents: HashMap<String, Box<dyn Agent>>,
    pub state: Arc<RwLock<EnvironmentState>>,
    pub status: SessionStatus,
}
```

#### 4.4 快照系统
**文件**: `crates/engine/src/session/snapshot.rs`
- 自动快照 (每个 Step 前)
- 回滚机制

#### 4.5 示例 Environment
**文件**:
- `crates/engine/src/environment/examples/chess.rs`
- `crates/engine/src/environment/examples/debate.rs`

**里程碑**: 可创建和管理 Session，支持暂停/恢复

---

### Phase 5: Engine 逻辑 (第 13-16 周)

#### 5.1 Local Agent
**文件**: `crates/engine/src/agent/local.rs`
- 环境原生 Agent
- 强耦合 Environment Schema
- 支持角色扮演

#### 5.2 Universal Agent (可复用服务)
**文件**: `crates/engine/src/agent/universal.rs`
- 与 Brain 中的 Universal Agent 同一套实现
- 可挂载到任意 Session

#### 5.3 Oracle Protocol
**文件**: `crates/engine/src/agent/oracle.rs`
```rust
pub struct OracleRequest {
    pub target_agent: String,
    pub query: String,
}
// Local Agent 通过 need_help 字段请求 Universal Agent 帮助
```

#### 5.4 Workflow Engine 核心
**文件**: `crates/engine/src/workflow/engine.rs`
```rust
pub struct WorkflowEngine {
    pub rhai_engine: rhai::Engine,
    pub session: Arc<Session>,
    pub oracle: Arc<OracleProtocol>,
}
// 注册 system.step(), agent(), env() 等函数
```

#### 5.5 Step 追踪
**文件**: `crates/engine/src/workflow/step.rs`
- Step 树状结构
- 状态追踪 (Pending/Running/Success/Failed)
- 前端可视化 JSON

#### 5.6 沙箱与资源限制
**文件**: `crates/engine/src/workflow/sandbox.rs`
```rust
pub struct SandboxLimits {
    pub max_instructions: u64,
    pub max_memory_mb: usize,
    pub timeout_sec: u64,
}
// 使用 Rhai 的限制 API + tokio::time::timeout
```

#### 5.7 Workflow 示例
**文件**:
- `examples/workflows/chess_game.rhai`
- `examples/workflows/news_debate.rhai`

**里程碑**: 可运行完整模拟，支持 Oracle 协议调用

---

### Phase 6: Synergy 协同 (第 17-18 周)

#### 6.1 Agent Registry
**文件**: `crates/synergy/src/registry/mod.rs`
```rust
pub struct AgentRegistry {
    agents: HashMap<String, AgentDefinition>,
    storage: Arc<dyn RegistryStorage>,  // PostgreSQL
}

impl AgentRegistry {
    pub async fn register(&self, def: AgentDefinition) -> Result<()>;
    pub fn instantiate(&self, def: &AgentDefinition) -> Result<Box<dyn Agent>>;
}
```

#### 6.2 Mission Control
**文件**: `crates/synergy/src/scheduler/mod.rs`
```rust
pub struct MissionControl {
    pub registry: Arc<AgentRegistry>,
    pub brain: Arc<BrainSystem>,
    pub engine: Arc<EngineSystem>,
}

pub enum TriggerType {
    Manual,
    Cron(String),
    Event(EventTrigger),
}
```

#### 6.3 定时调度
**文件**: `crates/synergy/src/scheduler/cron.rs`
- 使用 `tokio::time::interval` 实现简单调度
- 或使用 `tokio-cron-scheduler` crate

**里程碑**: 可注册 Agent，调度任务执行

---

### Phase 7: API 集成 ✅ 已完成

**状态**: 20个API端点全部实现，测试通过

#### 7.1 Brain API ✅
**文件**: `crates/api/src/main.rs`
```
GET    /api/brain/knowledge
POST   /api/brain/knowledge
DELETE /api/brain/knowledge/:id
GET    /api/brain/health
```

#### 7.2 Engine API ✅
**文件**: `crates/api/src/main.rs`
```
GET    /api/engine/sessions
POST   /api/engine/sessions
GET    /api/engine/sessions/:id
POST   /api/engine/sessions/:id/start
POST   /api/engine/sessions/:id/pause
POST   /api/engine/sessions/:id/stop
DELETE /api/engine/sessions/:id
GET    /api/engine/environments
```

#### 7.3 Synergy API ✅
**文件**: `crates/api/src/main.rs`
```
GET    /api/synergy/agents
POST   /api/synergy/agents
DELETE /api/synergy/agents/:id
GET    /api/synergy/tasks
POST   /api/synergy/tasks
DELETE /api/synergy/tasks/:id
```

#### 7.4 System API ✅
**文件**: `crates/api/src/main.rs`
```
GET /api/health
GET /api/version
GET /api/docs
GET /api/dashboard/stats
```

#### 7.5 测试覆盖 ✅
**文件**: `crates/api/tests/`
- `health_test.rs` - 4个测试
- `brain_api_test.rs` - 4个测试
- `engine_api_test.rs` - 5个测试
- `synergy_api_test.rs` - 7个测试

**测试结果**: ✅ 20个集成测试全部通过

**里程碑**: ✅ REST API 完整实现，前端可正常调用

---

## 关键技术决策

| 问题 | 决策 |
|------|------|
| LLM 集成 | 先用 Mock，后续添加 `async-openai` |
| Vector DB | 先用 in-memory cosine similarity，后续切换 Qdrant |
| Graph DB | 先用 `petgraph` in-memory，后续切换 Neo4j |
| 沙箱引擎 | Rhai (已配置)，配合资源限制 |
| 快照存储 | 先用文件系统，后续可扩展 S3 |
| 注册存储 | PostgreSQL (已配置 SQLx) |

---

## 需要添加的依赖

```toml
# 在 workspace Cargo.toml 中添加:
reqwest = { version = "0.12", features = ["json"] }  # HTTP client
feed-rs = "2.1"                                      # RSS 解析
petgraph = "0.6"                                     # 图算法
tokio-cron-scheduler = "0.10"                        # Cron 调度 (可选)
```

---

## 里程碑验证

### Milestone 1 (Phase 1 完成) ✅: 存储可用
- 采集数据 → RawArchive → UnifiedMemory → 检索
- **验证**: `cargo test -p brain` 60+ 测试通过

### Milestone 2 (Phase 3 完成) ✅: Brain 可用
- RSS 新闻 → Cleaner → Extractor → Tagger → 搜索 "最新AI新闻"
- **验证**: 示例程序 `brain_rss_pipeline.rs` 可运行

### Milestone 3 (Phase 5 完成) ✅: Engine 可用
- 创建 Chess Session → Workflow脚本执行 → 步骤追踪
- **验证**: `cargo test -p engine` 14个测试通过，Workflow引擎可执行Rhai脚本

### Milestone 4 (Phase 6 部分完成) ⚠️: 协同基础可用
- 注册 Agent → 创建 Mission → 任务管理
- **状态**: 基础API完成，需完善实际执行逻辑
- **验证**: `cargo test -p synergy` 5个测试通过

### Milestone 5 (Phase 7 完成) ✅: API集成完成
- 启动所有服务 → 20个API端点 → 前端可访问 → 测试通过
- **验证**: `cargo test -p api` 20个集成测试全部通过
- **访问**: http://localhost:8080/api/docs

---

## 用户确认的决策

1. **实施顺序**: 按计划顺序 (Phase 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7)
2. **LLM 方案**: 抽象层支持多厂商 (OpenAI, Ollama, 其他)

## 技术实现细节补充

### LLM 抽象层设计
**文件**: `crates/common/src/llm/mod.rs`
```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

// 实现版本
pub struct OpenAiClient { /* ... */ }
pub struct OllamaClient { /* ... */ }
pub struct MockClient { /* 用于测试 */ }
```

### Phase 0 即将实施的核心文件

#### 第一批文件 (测试与配置)
- `crates/common/src/testing/mod.rs` - 测试工具
- `crates/common/src/config/mod.rs` - 配置系统
- `crates/common/src/llm/mod.rs` - LLM 抽象层

#### 第二批文件 (Brain 存储)
- `crates/brain/src/storage/memory.rs` - 存储 Trait
- `crates/brain/src/storage/hot_memory.rs` - Redis 实现
- `crates/brain/src/storage/unified_memory.rs` - 统一 Facade

## 下一步行动 (已更新)

### 已完成 ✅
1. ✅ Phase 0-7 全部实现
2. ✅ 20个API端点，全部测试通过
3. ✅ Workflow引擎可运行Rhai脚本
4. ✅ 前端7个页面功能完整

### 当前优先级 (短期)

#### 1. 完善Synergy调度器 (1-2周)
**文件**: `crates/synergy/src/scheduler/mod.rs`
- [ ] 添加Cron调度循环 (tokio::time::interval)
- [ ] 实现Mission实际执行逻辑 (调用Engine Workflow)
- [ ] Agent实例化 (从定义到可运行实例)

```rust
// 需要实现
pub async fn start_scheduler(&self) {
    // 定时检查Cron任务
    // 执行到期的Mission
}
```

#### 2. 添加WebSocket支持 (1周)
**文件**: `crates/api/src/main.rs`
- [ ] Session状态实时推送
- [ ] Mission执行进度推送
- [ ] 系统事件广播

```rust
// 需要添加
.route("/ws/sessions/:id", get(session_ws_handler))
```

#### 3. 外部服务集成 (2-4周)
- [ ] Redis Hot Memory (修复API适配问题)
- [ ] Qdrant Vector Store (替换InMemoryVectorStore)
- [ ] Neo4j Graph Store (替换InMemoryGraphStore)

### 中期目标 (1-2个月)

#### 4. 前端功能增强
- [ ] Workflow编辑器执行功能 (连接API)
- [ ] 知识图谱可视化 (D3.js或React Flow)
- [ ] 实时监控面板 (WebSocket数据)

#### 5. 生产部署优化
- [ ] Docker Compose配置优化
- [ ] K8s部署配置
- [ ] 监控告警 (Prometheus + Grafana)

### 验证命令

```bash
# 验证所有测试
cargo test --workspace

# 验证编译
cargo build --release

# 启动服务验证
cargo run -p api
# 访问: http://localhost:8080/api/docs
```
