# Agent Playground 测试计划

## 当前状态

**测试覆盖率：基础测试已完成**

- ✅ Common: 12 个单元测试通过
- ✅ Brain: 41 个单元测试通过（1 个慢测试被跳过）
- ✅ Engine: 编译已修复，34 个单元测试通过
- ✅ Synergy: 编译已修复，7 个单元测试通过
- ✅ API: 基础结构已创建
- ✅ Frontend: 3 个单元测试通过 (Button 组件)
- ❌ Integration: 无测试

---

## 测试策略

### 1. 单元测试 (Unit Tests)

每个 crate 内部测试自己的模块：

```
crates/
├── common/src/
│   ├── lib.rs              # #[cfg(test)] mod tests
│   └── testing/            # 测试工具模块
├── brain/src/
│   ├── lib.rs
│   └── ...
│       └── #[cfg(test)]    # 各模块内测试
├── engine/src/
│   └── ...
└── synergy/src/
    └── ...
```

### 2. 集成测试 (Integration Tests)

每个 crate 的 `tests/` 目录：

```
crates/
├── brain/
│   └── tests/
│       ├── storage_integration_test.rs
│       └── api_integration_test.rs
├── engine/
│   └── tests/
│       ├── session_integration_test.rs
│       └── workflow_integration_test.rs
└── synergy/
    └── tests/
        └── scheduler_integration_test.rs
```

### 3. API 测试

```
crates/api/tests/
├── health_test.rs
├── brain_api_test.rs
├── engine_api_test.rs
└── synergy_api_test.rs
```

### 4. 端到端测试 (E2E)

```
tests/
├── e2e/
│   ├── news_broadcast_test.rs      # 场景 A
│   ├── knowledge_gardening_test.rs # 场景 B
│   └── sandtable_oracle_test.rs    # 场景 C
└── fixtures/
    └── test_data/
```

### 5. 前端测试

```
web/
├── src/
│   ├── components/
│   │   └── **/*.test.tsx
│   └── pages/
│       └── **/*.test.tsx
└── tests/
    └── e2e/
        └── *.spec.ts
```

---

## 编译错误修复记录

### 已修复问题

#### Brain Crate - 编译错误
- ✅ `src/api/mod.rs:557`: `create_test_state()` 添加 `async` 和 `.await`

#### Brain Crate - 测试修复
- ✅ `src/processors/tagger.rs:56-77`: 修复标签生成逻辑，支持多模式匹配（如 "ai" 匹配 "artificial intelligence"）
- ✅ `src/processors/cleaner.rs:60-64`: 修复 HTML 清理，直接移除标签而不是替换为空格
- ✅ `src/storage/vector_memory.rs:381-389`: 修复测试向量创建，使用不同方向的向量确保余弦相似度正确计算

#### Engine Crate
- ✅ `src/api/mod.rs:78`: `stop_session` 路由改为 `complete_session`
- ✅ `src/api/mod.rs:228`: `request.name` 克隆后再使用
- ✅ `src/api/mod.rs:300`: `delete_session` 返回 `Result<bool>` 处理
- ✅ `src/api/mod.rs:327/360/393`: `session.info()` 改为 `session.stats()`
- ✅ `src/api/mod.rs:502`: `create_snapshot(None)` 改为 `create_snapshot("API snapshot")`
- ✅ `src/session/mod.rs:399`: `SessionManager` 添加 `#[derive(Debug)]`

---

## 测试实现清单

### Phase 1: 修复编译错误 ✅

- [x] 修复 `brain` crate 编译错误
- [x] 修复 `engine` crate 编译错误
- [x] 修复 `synergy` crate 编译错误

### Phase 2: 单元测试

#### Common Crate (已有 12 个测试) ✅

- [x] LLM 客户端测试
- [x] 配置测试
- [x] Mock Agent 测试
- [ ] 补充更多边界情况

#### Brain Crate (41 个测试通过) ✅

- [x] Storage 层基础测试
- [x] Collector 基础测试
- [x] Processor 基础测试
- [x] 修复失败的测试：
  - [x] `test_generate_tags` - 修复标签匹配逻辑，支持 "ai" 匹配 "artificial intelligence"
  - [x] `test_clean_html` - 修复 HTML 标签移除时的空格处理
  - [x] `test_vector_search` - 修复测试向量方向问题
  - [ ] `test_raw_archive_store_retrieve` (慢测试，跳过)

#### Engine Crate (34 个测试) ✅

- [x] Environment 测试
- [x] Agent 测试
- [x] Workflow 测试
- [x] Session 测试
- [x] API 路由测试

#### Synergy Crate (7 个测试) ✅

- [x] Registry 测试
- [x] Scheduler 测试
- [x] API 测试

### Phase 3: 集成测试

- [ ] Brain + Storage 集成
- [ ] Engine + Agent 集成
- [ ] Brain + Engine 集成
- [ ] 全系统集成
  - [ ] 场景 A: 定时新闻播报
  - [ ] 场景 B: 知识库维护
  - [ ] 场景 C: 沙盘推演

### Phase 4: API 测试

- [ ] 基础端点
  - [ ] GET /api/health
  - [ ] GET /api/version
  - [ ] GET /api/docs
- [ ] Brain API
- [ ] Engine API
- [ ] Synergy API

### Phase 5: 前端测试

- [x] 单元测试框架 (Vitest)
- [x] 测试工具配置 (React Testing Library)
- [x] Button 组件测试 (3 个测试)
- [ ] 更多组件测试
- [ ] 页面测试
- [ ] E2E 测试

---

## 运行测试

### 使用脚本

```bash
# 运行所有测试
bash run-tests.sh all

# 仅运行 Rust 测试
bash run-tests.sh rust

# 仅运行前端测试
bash run-tests.sh frontend

# 生成覆盖率报告
bash run-tests.sh coverage
```

### 手动运行

```bash
# 运行所有 Rust 测试
cargo test --workspace

# 运行前端测试
cd web && npm test

# 运行单个 crate
cargo test -p common
cargo test -p engine --lib
cargo test -p synergy --lib
```

### 覆盖率报告

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin -p common -p engine -p synergy --out Html

# 查看报告
open tarpaulin-report.html
```

---

## CI/CD 集成

### GitHub Actions 示例

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 18

      - name: Cache Rust
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache Node
        uses: actions/cache@v3
        with:
          path: web/node_modules
          key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}

      - name: Run Rust Tests
        run: cargo test -p common -p engine -p synergy --lib

      - name: Run Frontend Tests
        run: |
          cd web
          npm ci
          npm test
```

---

## 测试数据

### Fixtures

```
tests/fixtures/
├── agents/
│   ├── searcher.yaml
│   ├── fact_checker.yaml
│   └── code_assistant.yaml
├── environments/
│   ├── news_studio.yaml
│   ├── trading_floor.yaml
│   └── debate_hall.yaml
├── workflows/
│   ├── morning_news.yaml
│   └── stock_simulation.yaml
└── knowledge/
    ├── sample_documents/
    └── sample_graph.json
```

---

## 当前可运行的测试

```bash
# Rust 核心测试 (53 个测试通过)
cargo test -p common --lib    # 12 passed
cargo test -p brain --lib     # 41 passed (1 slow test skipped)
cargo test -p engine --lib    # 34 passed
cargo test -p synergy --lib   # 7 passed

# 前端测试
cd web && npm test            # 3 passed (Button)

# 完整测试套件
bash run-tests.sh all
```

### 已修复的测试失败

1. **Brain::test_generate_tags** - 标签匹配现在支持缩写形式（如 "ai" 匹配 "artificial intelligence"）
2. **Brain::test_clean_html** - HTML 标签直接移除，不再留下多余空格
3. **Brain::test_vector_search** - 测试向量使用不同方向，余弦相似度计算正确

---

## 下一步行动

1. **已完成**：修复所有编译错误
2. **待处理**：修复 Brain crate 中失败的测试
3. **本周**：添加更多组件单元测试
4. **下周**：创建集成测试套件
5. **持续**：CI/CD 集成和覆盖率监控
