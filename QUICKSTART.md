# Agent Playground - 快速开始指南

## 概述

Agent Playground 采用单体程序架构，所有的服务（包含存储）都作为嵌入式服务运行在单一的进程中。

## 环境要求

- Node.js 18+
- Rust 1.75+
- 4GB+ 内存
- 10GB+ 磁盘空间

## 快速开始

### 开发模式（推荐用于开发）

```bash
# 1. 安装依赖（首次）
./manage.sh install

# 2. 一键启动所有服务（本地热重载）
./manage.sh dev
```

**说明**：`./manage.sh dev` 会：
1. 启动 API 服务（热重载，并包含嵌入式存储的启动）
2. 启动前端服务（热重载）

**停止服务**：
- 按 `Ctrl+C` 停止 API 和前端服务

服务启动后访问：
- API: http://localhost:8080
- 前端: http://localhost:5173
- API 文档: http://localhost:8080/api/docs

## 命令结构

### 核心命令

```bash
# 开发模式 - 本地进程启动所有服务（热重载）
./manage.sh dev
```

### 项目命令

```bash
# 构建
./manage.sh build             # 开发模式构建
./manage.sh build release     # 生产模式构建

# 运行（单独运行 API）
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
# 单独运行 API（手动控制）
./manage.sh run

# 提示：你也可以在前端目录内使用你熟悉的前端命令启动
```

### 3. 停止服务

```bash
# 停止开发模式（Ctrl+C）
```

## 服务访问

### 访问地址

| 服务 | 地址 | 说明 |
|------|------|------|
| API | http://localhost:8080 | REST API |
| 前端 | http://localhost:5173 | Vite 开发服务器 |
| API 文档 | http://localhost:8080/api/docs | Swagger 文档 |
| 健康检查 | http://localhost:8080/api/health | 健康状态 |

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
lsof -i :5173

# 修改 .env 中的端口配置
```

## 数据持久化

### 数据存储

数据将通过嵌入式数据库（如 SQLite）持久化在本地目录中，具体可以参考配置项指定的路径。

### 清理数据

如果需要清理数据，直接删除对应的存储目录即可。

## 下一步

- 阅读 开发文档 了解详细选项
- 查看 [API 文档](http://localhost:8080/api/docs)（启动服务后）

## 帮助

```bash
./manage.sh help      # 管理脚本帮助
```
