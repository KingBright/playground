# Agent Playground - 部署指南

## 概述

本文档提供 Agent Playground 的多种部署方式，包括：
- Docker Compose 一键部署（推荐开发和测试）
- Kubernetes 部署（推荐生产环境）
- 本地开发部署

## 快速开始

### 方式一：Docker Compose 一键部署（推荐）

```bash
# 1. 克隆项目
git clone <your-repo-url>
cd playground

# 2. 一键部署生产环境
./deploy.sh prod

# 3. 等待服务启动完成后访问：
# API: http://localhost:8080
# API 文档: http://localhost:8080/api/docs
```

### 方式二：开发模式

```bash
# 只启动数据库等基础设施
./deploy.sh dev

# 本地运行 API
./manage.sh run
```

## 部署架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Docker Compose                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐                                                           │
│  │  API (Rust)  │  Port: 8080                                               │
│  └──────┬───────┘                                                           │
│         │                                                                   │
│  ┌──────┴───────┬─────────────────┬─────────────────┐                       │
│  │              │                 │                 │                       │
│  ▼              ▼                 ▼                 ▼                       │
│ ┌──────┐   ┌──────────┐   ┌───────────┐   ┌─────────────┐                  │
│ │Redis │   │PostgreSQL│   │  Qdrant   │   │    Neo4j    │                  │
│ │:6379 │   │  :5432   │   │  :6333    │   │   :7474     │                  │
│ └──────┘   └──────────┘   └───────────┘   └─────────────┘                  │
│                                                                ┌─────────┐  │
│                                                                │  MinIO  │  │
│                                                                │  :9000  │  │
│                                                                └─────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Docker Compose 部署

### 环境要求

- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ 可用内存
- 10GB+ 磁盘空间

### 部署命令

```bash
# 查看帮助
./deploy.sh help

# 生产部署（完整部署）
./deploy.sh prod

# 开发模式（只启动数据库）
./deploy.sh dev

# 启动基础服务
./deploy.sh up

# 启动包含前端的服务
./deploy.sh up web

# 查看状态
./deploy.sh status

# 查看日志
./deploy.sh logs
./deploy.sh logs api

# 停止服务
./deploy.sh down

# 重启服务
./deploy.sh restart

# 完全清理（包括数据）
./deploy.sh clean

# 备份数据
./deploy.sh backup
```

### 配置环境变量

复制 `.env.example` 为 `.env` 并修改：

```bash
cp .env.example .env
```

主要配置项：

| 变量 | 默认 | 说明 |
|------|------|------|
| API_PORT | 8080 | API 服务端口 |
| RUST_LOG | info | 日志级别 |
| POSTGRES_PASSWORD | postgres | 数据库密码 |
| NEO4J_PASSWORD | password | Neo4j 密码 |
| MINIO_ROOT_PASSWORD | minioadmin | MinIO 密码 |

### 数据持久化

数据通过 Docker Volumes 持久化：

- `postgres-data` - PostgreSQL 数据
- `redis-data` - Redis 数据
- `qdrant-data` - Qdrant 向量数据
- `neo4j-data` - Neo4j 图数据
- `minio-data` - MinIO 对象存储

### 服务访问

| 服务 | 地址 | 默认凭据 |
|------|------|----------|
| API | http://localhost:8080 | - |
| API Docs | http://localhost:8080/api/docs | - |
| Health | http://localhost:8080/api/health | - |
| Neo4j | http://localhost:7474 | neo4j/password |
| MinIO | http://localhost:9001 | minioadmin/minioadmin |

## Kubernetes 部署

### 前置要求

- Kubernetes 1.24+
- kubectl
- kustomize (可选)

### 使用 Kustomize 部署

```bash
# 部署所有资源
kubectl apply -k k8s/

# 查看部署状态
kubectl get pods -n agent-playground

# 查看日志
kubectl logs -n agent-playground -l app=agent-playground-api

# 端口转发访问 API
kubectl port-forward -n agent-playground svc/agent-playground-api 8080:8080

# 删除部署
kubectl delete -k k8s/
```

### 使用 Helm 部署（预留）

```bash
# 添加 Helm 仓库
helm repo add agent-playground https://your-repo.github.io/charts

# 安装
helm install agent-playground agent-playground/agent-playground

# 升级
helm upgrade agent-playground agent-playground/agent-playground
```

## 本地开发部署

### 1. 安装依赖

```bash
# 安装 Rust 依赖和前端依赖
./manage.sh install
```

### 2. 启动基础设施

```bash
# 使用 Docker Compose 启动数据库
./deploy.sh dev
```

### 3. 本地运行 API

```bash
# 运行 API 服务
./manage.sh run

# 或开发模式（热重载）
./manage.sh dev
```

### 4. 本地运行前端

```bash
cd web
npm install
npm run dev
```

## 生产环境部署建议

### 1. 数据库高可用

- PostgreSQL: 使用主从复制或云数据库服务
- Redis: 使用 Redis Cluster 或云 Redis 服务
- Qdrant: 使用分布式部署
- Neo4j: 使用 Neo4j Cluster
- MinIO: 使用分布式 MinIO 或云存储服务

### 2. API 服务扩展

```yaml
# 修改 k8s/api-deployment.yaml
spec:
  replicas: 3  # 根据负载调整
```

### 3. 监控和日志

建议添加以下组件：
- Prometheus + Grafana - 监控
- ELK Stack 或 Loki - 日志收集
- Jaeger 或 Zipkin - 链路追踪

### 4. 安全配置

- 使用 HTTPS
- 配置防火墙规则
- 使用 Secrets 管理敏感信息
- 定期备份数据

## 故障排查

### 服务无法启动

```bash
# 检查端口占用
lsof -i :8080

# 查看 Docker 日志
docker-compose logs

# 检查资源使用
docker stats
```

### 数据库连接失败

```bash
# 检查数据库健康状态
docker-compose ps

# 查看数据库日志
docker-compose logs postgres
```

### 内存不足

```bash
# 调整 Docker 资源限制
# 修改 docker-compose.yml 中的 deploy.resources 部分
```

## 更新部署

```bash
# 拉取最新代码
git pull origin main

# 更新部署
./deploy.sh update
```

## 备份和恢复

### 备份

```bash
./deploy.sh backup
```

备份文件保存在 `./backups/YYYYMMDD_HHMMSS/` 目录。

### 恢复

```bash
# 1. 停止服务
./deploy.sh down

# 2. 恢复数据卷（根据备份类型）
# PostgreSQL
docker exec -i agent-playground-postgres-1 psql -U postgres agent_platform < backup.sql

# Redis
docker cp redis.rdb agent-playground-redis-1:/data/dump.rdb

# 3. 重启服务
./deploy.sh up
```

## 性能调优

### API 服务

```yaml
# docker-compose.yml
services:
  api:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
```

### 数据库

```yaml
# Neo4j 内存配置
services:
  neo4j:
    environment:
      - NEO4J_dbms_memory_heap_max__size=2G
```

## 安全建议

1. **修改默认密码**: 修改所有服务的默认密码
2. **使用 HTTPS**: 生产环境必须使用 HTTPS
3. **网络隔离**: 使用 Docker network 或 K8s namespace 隔离服务
4. **定期更新**: 定期更新基础镜像和依赖
5. **访问控制**: 配置适当的防火墙规则

## 支持

- 查看日志: `./deploy.sh logs`
- 检查状态: `./deploy.sh status`
- 提交 Issue: [GitHub Issues](https://github.com/your-repo/issues)
