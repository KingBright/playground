# Agent Playground - 多 Agent 协同仿真平台

基于 Rust + TypeScript 构建的多 Agent 协同仿真平台，包含三大核心系统：

- **External Brain**: 知识供应链系统（采集、加工、存储）
- **Agent Playground**: 基于 ECS 的仿真引擎
- **System Synergy**: 任务调度与协同中心

本平台采用**单体程序架构 (Monolithic Architecture)**和**嵌入式存储**方案，无需依赖外部的 Docker、Kubernetes 或独立运行的数据库服务，开箱即用。

## 快速开始

### 开发模式（推荐，3 分钟启动）

```bash
# 1. 克隆项目
git clone <your-repo-url>
cd playground

# 2. 安装依赖（首次）
./manage.sh install

# 3. 启动所有服务（本地热重载）
./manage.sh dev

# 4. 访问服务
# API: http://localhost:8080
# 前端: http://localhost:5173
# 文档: http://localhost:8080/api/docs

# 5. 停止服务
# 按 Ctrl+C 停止 API 和前端
```

## 项目结构

```
playground/
├── crates/
│   ├── common/          # 共享类型和工具
│   ├── brain/           # External Brain 系统
│   │   ├── collectors/  # 数据采集器
│   │   ├── processors/  # Agent 加工层
│   │   └── storage/     # 存储层（Hot/Vector/Graph/Raw）
│   ├── engine/          # Agent Playground 仿真引擎
│   │   ├── environment/ # 环境定义
│   │   ├── agent/       # Agent 运行时
│   │   └── workflow/    # 工作流引擎
│   └── synergy/         # System Synergy 调度中心
│       ├── registry/    # Agent 注册表
│       └── scheduler/   # 任务调度器
├── web/                 # TypeScript 前端
└── manage.sh            # 项目管理脚本
```

## 管理命令

### 核心命令 (`./manage.sh`)

```bash
# 开发模式 - 本地进程启动所有服务（热重载）
./manage.sh dev
```

### 项目命令 (`./manage.sh`)

```bash
./manage.sh build         # 构建项目
./manage.sh run           # 单独运行 API
./manage.sh test          # 运行测试
./manage.sh lint          # 代码检查
./manage.sh fmt           # 代码格式化
./manage.sh check         # 完整检查
./manage.sh install       # 安装依赖
./manage.sh clean         # 清理构建产物
```

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust, Tokio, Axum |
| 前端 | React, TypeScript, Vite |
| 脚本 | Rhai |
| 存储 | 嵌入式方案 (如 SQLite, in-memory) |

## 系统架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Agent Playground                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                      │
│  │   External   │  │    Agent     │  │    System    │                      │
│  │    Brain     │  │   Playground │  │   Synergy    │                      │
│  │              │  │              │  │              │                      │
│  │ • Collectors │  │ • Environment│  │ • Registry   │                      │
│  │ • Processors │  │ • Agents     │  │ • Scheduler  │                      │
│  │ • Storage    │  │ • Workflow   │  │ • Missions   │                      │
│  └──────────────┘  └──────────────┘  └──────────────┘                      │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                           Embedded Data Storage Layer                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐          │
│  │  Hot    │  │  Vector  │  │  Graph  │  │   Raw   │  │  Meta   │          │
│  └─────────┘  └──────────┘  └─────────┘  └─────────┘  └─────────┘          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 文档

- [快速开始指南](./QUICKSTART.md)
- [开发文档](./DEVELOPMENT.md)
- [API 文档](http://localhost:8080/api/docs) (启动服务后访问)

## 环境要求

### 本地开发
- Rust 1.75+
- Node.js 18+

## 默认端口

| 服务 | 端口 | 说明 |
|------|------|------|
| API | 8080 | REST API 服务 |
| Web | 3000 | 前端开发服务器 |

## 默认凭据

| 服务 | 用户名 | 密码 |
|------|--------|------|

## 测试

```bash
# 运行所有测试
./manage.sh test

# 运行 Rust 测试
./manage.sh test rust

# 生成测试覆盖率报告
./manage.sh test coverage
```

## 贡献

1. Fork 项目
2. 创建分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

[MIT](LICENSE) OR [Apache-2.0](LICENSE)
