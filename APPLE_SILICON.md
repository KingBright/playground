# Apple Silicon (M1/M2/M3) 支持指南

## 概述

Agent Playground 完全支持 Apple Silicon (ARM64) 架构，包括 M1、M2、M3 系列芯片。

## 自动检测

启动脚本会自动检测你的系统架构：

```bash
./manage.sh services start
```

输出示例（Apple Silicon）：
```
[INFO] 系统架构: arm64
[INFO] Docker 平台: linux/arm64
[INFO] 检测到 Apple Silicon (M1/M2/M3)
[INFO] 将使用 ARM64 架构镜像以获得最佳性能
```

## 支持的镜像

所有基础服务镜像都支持多架构（AMD64 和 ARM64）：

| 服务 | 镜像 | Apple Silicon 支持 |
|------|------|-------------------|
| PostgreSQL | postgres:15-alpine | ✅ ARM64 原生 |
| Redis | redis:7-alpine | ✅ ARM64 原生 |
| Qdrant | qdrant/qdrant:latest | ✅ ARM64 原生 |
| Neo4j | neo4j:5-community | ✅ ARM64 原生 |
| MinIO | minio/minio:latest | ✅ ARM64 原生 |

## 性能优势

在 Apple Silicon 上使用 ARM64 镜像相比 x86 模拟：

- **启动速度**: 提升 2-3 倍
- **内存占用**: 减少 20-30%
- **CPU 效率**: 提升 40-60%
- **整体体验**: 更流畅，更省电

## 手动指定架构

如果你需要强制使用特定架构，可以设置环境变量：

```bash
# 强制使用 ARM64（Apple Silicon）
export DOCKER_PLATFORM=linux/arm64
./manage.sh services start

# 强制使用 AMD64（Intel/AMD）
export DOCKER_PLATFORM=linux/amd64
./manage.sh services start
```

或者在 `.env` 文件中设置：

```bash
# .env
DOCKER_PLATFORM=linux/arm64
```

## 验证架构

验证容器是否运行在正确的架构上：

```bash
# 查看容器架构
docker exec playground-redis-1 uname -m
# 输出: aarch64 (ARM64) 或 x86_64 (AMD64)

# 查看所有服务架构
docker ps --format "table {{.Names}}\t{{.Image}}" | grep playground
```

## 常见问题

### 1. 镜像拉取慢

首次在 Apple Silicon 上运行时，Docker 需要下载 ARM64 版本的镜像：

```bash
# 正常现象，等待下载完成
[INFO] 系统架构: arm64
[INFO] 将使用 ARM64 架构镜像以获得最佳性能
[+] Running 5/5
 ⠧ Image postgres:15-alpine   Pulling  # 下载 ARM64 版本
```

**解决方法**: 耐心等待首次下载，后续启动会秒开。

### 2. 混合架构问题

如果你之前用 Rosetta 运行过 x86 镜像，可能需要清理：

```bash
# 停止所有服务
./manage.sh services stop

# 删除旧镜像（强制重新下载正确架构）
docker rmi redis:7-alpine postgres:15-alpine qdrant/qdrant:latest neo4j:5-community minio/minio:latest

# 重新启动
./manage.sh services start
```

### 3. 构建 API 镜像（生产模式）

在 Apple Silicon 上构建生产镜像时，会自动使用 ARM64 基础镜像：

```bash
./deploy.sh prod
# 输出:
# [INFO] 系统架构: arm64
# [INFO] 将为 Apple Silicon 构建 ARM64 镜像
```

## 架构对比

| 特性 | x86 模拟 (Rosetta) | ARM64 原生 |
|------|-------------------|-----------|
| 启动时间 | 慢 (10-20s) | 快 (2-5s) |
| 内存占用 | 高 | 低 |
| CPU 使用率 | 高 | 低 |
| 电池续航 | 耗电快 | 正常 |
| 兼容性 | 可能有 issue | 完美支持 |

## 故障排查

### 检查 Docker Desktop 设置

确保 Docker Desktop 启用了 Apple Silicon 支持：

1. 打开 Docker Desktop
2. 进入 Settings (设置) → General
3. 确认 "Use Rosetta for x86/amd64 emulation" 选项（可选，但建议开启作为后备）

### 验证多架构支持

```bash
# 检查镜像是否支持多架构
docker manifest inspect redis:7-alpine | grep architecture

# 应该看到多种架构，包括 arm64
```

### 强制重新拉取正确架构

```bash
# 删除特定平台的镜像缓存
./manage.sh services stop
docker compose -f docker-compose.yml down --rmi all
./manage.sh services start
```

## 相关命令

```bash
# 查看系统架构
uname -m  # 输出: arm64

# 查看 Docker 信息
docker version

# 查看容器使用的架构
docker inspect playground-redis-1 --format '{{.Os}}/{{.Architecture}}'

# 启动服务（自动检测架构）
./manage.sh services start
./deploy.sh prod
```

## 参考

- [Docker Apple Silicon 文档](https://docs.docker.com/desktop/mac/apple-silicon/)
- [Apple Silicon 容器化最佳实践](https://medium.com/@cyborg.m/coding-on-apple-silicon-m1-m2-docker-and-nodejs-6e82fbf4a8bb)
