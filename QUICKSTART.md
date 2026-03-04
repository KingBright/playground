# Agent Playground - 快速开始指南

## 概述

Agent Playground 提供两种对等的部署模式：

| 模式 | 命令 | 说明 |
|------|------|------|
| **开发模式** | `./manage.sh dev` | 本地进程启动所有服务，支持热重载 |
| **生产模式** | `./manage.sh prod` | Docker 启动所有服务，优化配置 |

## 环境要求

- Docker 20.10+ （用于数据库服务）
- Docker Compose 2.0+ （可选，用于生产部署）
- Node.js 18+
- Rust 1.75+
- 4GB+ 内存
- 10GB+ 磁盘空间

## 快速开始

### 方式一：开发模式（推荐用于开发）

```bash
# 1. 安装依赖（首次）
./manage.sh install

# 2. 一键启动所有服务（本地热重载）
./manage.sh dev
```

**说明**：`./manage.sh dev` 会：
1. 启动基础设施服务（数据库等，后台运行）
2. 启动 API 服务（热重载）
3. 启动前端服务（热重载）

**停止服务**：
- 按 `Ctrl+C` 停止 API 和前端服务
- 基础设施（数据库）继续在后台运行
- 如需停止基础设施：`./manage.sh services stop`

服务启动后访问：
- API: http://localhost:8080
- 前端: http://localhost:5173
- API 文档: http://localhost:8080/api/docs

### 方式二：生产模式（推荐用于部署）

```bash
# 一键 Docker 部署
./manage.sh prod

# 或使用 deploy.sh
./deploy.sh prod
```

访问：
- API: http://localhost:8080
- API 文档: http://localhost:8080/api/docs

## 命令结构

### 核心命令

```bash
# 开发模式 - 本地进程启动所有服务（热重载）
./manage.sh dev

# 生产模式 - Docker 启动所有服务
./manage.sh prod

./manage.sh services start    # 启动基础设施
./manage.sh services stop     # 停止基础设施
./manage.sh services status   # 查看状态
./manage.sh services logs     # 查看日志
```

### 项目命令

```bash
# 构建
./manage.sh build             # 开发模式构建
./manage.sh build release     # 生产模式构建

# 运行（单独运行 API，需要基础设施已启动）
./manage.sh run

# 测试
./manage.sh test              # 运行所有测试
./manage.sh test rust         # 运行 Rust 测试
./manage.sh test coverage     # 生成覆盖率报告

# 代码质量
./manage.sh check             # 完整检查（fmt + lint + test）
./manage.sh fmt               # 格式化代码
./manage.sh lint              # 代码检查

# 清理
./manage.sh clean             # 清理构建产物
```

## 常见工作流

### 1. 首次开发

```bash
# 安装依赖
./manage.sh install

# 启动所有服务（开发模式）
./manage.sh dev
```

### 2. 日常开发

如果你只想控制特定部分：

```bash
# 只启动基础设施（数据库等）
./manage.sh services start

# 单独运行 API（手动控制）
./manage.sh run

# 单独运行前端（新终端）
cd web && npm run dev
```

### 3. 查看日志

```bash
# 查看基础设施日志
./manage.sh services logs

# 生产模式查看日志
./deploy.sh logs
./deploy.sh logs api
```

### 4. 停止服务

```bash
# 停止开发模式（Ctrl+C）

# 停止基础设施
./manage.sh services stop

# 停止生产模式
./deploy.sh down
```

## 服务访问

### 开发模式

| 服务 | 地址 | 说明 |
|------|------|------|
| API | http://localhost:8080 | REST API |
| 前端 | http://localhost:5173 | Vite 开发服务器 |
| API 文档 | http://localhost:8080/api/docs | Swagger 文档 |

### 生产模式

| 服务 | 地址 | 说明 |
|------|------|------|
| API | http://localhost:8080 | REST API |
| API 文档 | http://localhost:8080/api/docs | Swagger 文档 |
| 健康检查 | http://localhost:8080/api/health | 健康状态 |

### 数据库服务（两种模式通用）

| 服务 | 地址 | 默认凭据 |
|------|------|----------|
| Redis | localhost:6379 | - |
| Qdrant | http://localhost:6333 | - |

## 配置

### 环境变量

复制 `.env.example` 为 `.env` 并修改：

```bash
cp .env.example .env
```

主要配置项：

| 变量 | 默认 | 说明 |
|------|------|------|
| API_PORT | 8080 | API 服务端口 |
| RUST_LOG | info | 日志级别 (error/warn/info/debug/trace) |

## 故障排查

### 端口占用

```bash
# 检查端口占用
lsof -i :8080
lsof -i :5432
lsof -i :6379

# 修改 .env 中的端口配置
```

### 服务无法启动

```bash
# 查看基础设施状态
./manage.sh services status

# 查看日志
./manage.sh services logs

# 完全重置
./manage.sh services clean
./manage.sh services start
```

### Docker 问题

```bash
# 检查 Docker 状态

# 清理 Docker 缓存

# 重新构建镜像
./deploy.sh build
```

## 数据持久化

### 开发模式

数据通过 Docker Volumes 持久化，即使删除容器数据也不会丢失：


### 清理数据

```bash
# 清理基础设施数据（警告：会删除所有数据！）
./manage.sh services clean

# 生产模式清理
./deploy.sh clean
```

## 备份和恢复

### 备份

```bash
# 生产模式备份
./deploy.sh backup

# 备份文件保存在 ./backups/YYYYMMDD_HHMMSS/
```

### 恢复

```bash
# 1. 停止服务
./deploy.sh down

# 2. 恢复数据（根据备份类型）

# Redis

# 3. 重启服务
./deploy.sh up
```

## 下一步

- 阅读 [部署指南](./DEPLOYMENT.md) 了解更详细的部署选项
- 查看 [API 文档](http://localhost:8080/api/docs)（启动服务后）
- 探索 [开发文档](./DEVELOPMENT.md)

## 帮助

```bash
./manage.sh help      # 管理脚本帮助
./deploy.sh help      # 部署脚本帮助
```
