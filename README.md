# Agent Playground - 多 Agent 协同仿真平台

基于 Rust + TypeScript 构建的多 Agent 协同仿真平台，包含三大核心系统：

- **External Brain**: 知识供应链系统（采集、加工、存储）
- **Agent Playground**: 基于 ECS 的仿真引擎
- **System Synergy**: 任务调度与协同中心

## 快速开始

### 方式一：开发模式（推荐，3 分钟启动）

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
# 按 Ctrl+C 停止 API 和前端，数据库继续运行
# 如需停止数据库: ./manage.sh services stop
```

### 方式二：生产模式（Docker 部署）

```bash
# 一键 Docker 部署
./manage.sh prod

# 或直接使用 deploy.sh
./deploy.sh prod
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
├── docker-compose.yml   # Docker Compose 配置
├── deploy.sh            # 部署脚本
└── manage.sh            # 项目管理脚本
```

## 管理命令

### 核心命令 (`./manage.sh`)

```bash
# 开发模式 - 本地进程启动所有服务（热重载）
./manage.sh dev

# 生产模式 - Docker 启动所有服务
./manage.sh prod

# 基础设施管理（数据库等）
./manage.sh services start    # 启动基础设施
./manage.sh services stop     # 停止基础设施
./manage.sh services status   # 查看状态
./manage.sh services logs     # 查看日志
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

### Docker 部署 (`./deploy.sh`)

```bash
./deploy.sh prod          # 生产部署
./deploy.sh up            # 启动服务
./deploy.sh down          # 停止服务
./deploy.sh status        # 查看状态
./deploy.sh logs          # 查看日志
./deploy.sh backup        # 备份数据
```

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust, Tokio, Axum |
| 前端 | React, TypeScript, Vite |
| 脚本 | Rhai |
| 存储 | PostgreSQL, Redis, Qdrant, Neo4j, MinIO |
| 部署 | Docker, Docker Compose, Kubernetes |

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
│                           Data Storage Layer                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐          │
│  │  Hot    │  │  Vector  │  │  Graph  │  │   Raw   │  │  Meta   │          │
│  │ (Redis) │  │ (Qdrant) │  │ (Neo4j) │  │ (MinIO) │  │(Postgres)│         │
│  └─────────┘  └──────────┘  └─────────┘  └─────────┘  └─────────┘          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 文档

- [快速开始指南](./QUICKSTART.md)
- [部署指南](./DEPLOYMENT.md)
- [开发文档](./DEVELOPMENT.md)
- [API 文档](http://localhost:8080/api/docs) (启动服务后访问)

## 环境要求

### Docker 部署
- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ 内存
- 10GB+ 磁盘空间

### 本地开发
- Rust 1.75+
- Node.js 18+
- PostgreSQL 15+
- Redis 7+

## 默认端口

| 服务 | 端口 | 说明 |
|------|------|------|
| API | 8080 | REST API 服务 |
| Web | 3000 | 前端开发服务器 |
| PostgreSQL | 5432 | 关系型数据库 |
| Redis | 6379 | 缓存 |
| Qdrant | 6333 | 向量数据库 |
| Neo4j | 7474 | 图数据库浏览器 |
| MinIO | 9000 | 对象存储 API |
| MinIO Console | 9001 | 对象存储控制台 |

## 默认凭据

| 服务 | 用户名 | 密码 |
|------|--------|------|
| PostgreSQL | postgres | postgres |
| Neo4j | neo4j | password |
| MinIO | minioadmin | minioadmin |

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
