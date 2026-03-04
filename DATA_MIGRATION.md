# Agent Playground - 数据迁移指南

## 概述

本项目提供完整的数据迁移方案，支持一键导出、导入、备份所有数据。

## Redis 内存优化

### 默认配置

- **内存限制**: 256MB（可通过环境变量调整）
- **淘汰策略**: `allkeys-lru`（最近最少使用）
- **持久化**: RDB 格式（适合备份迁移）

### 修改内存限制

```bash
# 编辑 .env 文件
vim .env

# 修改 Redis 内存限制
REDIS_MEMORY_LIMIT=512m  # 根据你的服务器调整
```

可选值：
- `128m` - 最小配置，适合测试
- `256m` - 默认配置，适合开发（推荐）
- `512m` - 适中配置，适合小规模生产
- `1g` - 较大配置，适合生产环境

### 配置文件位置

Redis 配置文件: `config/redis/redis.conf`

## 数据迁移工具

### 快速开始

```bash
# 1. 创建备份（一键打包所有数据）
./data-migrate.sh backup

# 2. 查看备份列表
./data-migrate.sh list

# 3. 迁移到新机器
# 复制 backups/ 目录到新机器
# 然后恢复
./data-migrate.sh restore ./backups/agent-playground-backup-xxx.tar.gz
```

### 命令详解

#### 1. 导出数据

```bash
# 导出到默认目录（./data-export/YYYYMMDD_HHMMSS/）
./data-migrate.sh export

# 导出到指定目录
./data-migrate.sh export ./my-backup
```

导出内容包括：
- `postgres.sql` - PostgreSQL 数据库导出
- `redis.rdb` - Redis 持久化文件
- `qdrant.tar.gz` - Qdrant 向量数据
- `neo4j.tar.gz` - Neo4j 图数据
- `minio.tar.gz` - MinIO 对象存储
- `export-info.json` - 导出信息

#### 2. 导入数据

```bash
# 从目录导入
./data-migrate.sh import ./data-export/20240115_120000

# 从压缩包导入
./data-migrate.sh restore ./backups/agent-playground-backup-xxx.tar.gz
```

**注意**: 导入前请确保服务已启动。

#### 3. 创建压缩备份

```bash
# 创建带时间戳的压缩备份
./data-migrate.sh backup

# 备份文件位置: ./backups/agent-playground-backup-YYYYMMDD_HHMMSS.tar.gz
```

#### 4. 列出备份

```bash
./data-migrate.sh list
```

显示：
- 所有压缩备份文件
- 最近导出目录

#### 5. 清理数据

```bash
# 危险操作：删除所有数据！
./data-migrate.sh clean
```

需要输入 `yes` 确认。

### 完整迁移流程

#### 从旧机器迁移到新机器

**旧机器：**

```bash
# 1. 进入项目目录
cd /path/to/playground

# 2. 创建备份
./data-migrate.sh backup

# 3. 查看备份文件
ls -lh ./backups/
# 输出: agent-playground-backup-20240115_120000.tar.gz

# 4. 复制备份到新机器（使用 scp 或其他方式）
scp ./backups/agent-playground-backup-*.tar.gz user@new-server:/path/to/playground/backups/
```

**新机器：**

```bash
# 1. 克隆项目
git clone <your-repo-url>
cd playground

# 2. 启动 Docker（仅启动基础设施）
./manage.sh services start

# 3. 等待服务就绪
./manage.sh services status

# 4. 恢复数据
./data-migrate.sh restore ./backups/agent-playground-backup-20240115_120000.tar.gz

# 5. 启动应用
./manage.sh dev  # 或 ./manage.sh prod
```

## 数据存储说明

### PostgreSQL

- **用途**: 关系型数据（Agent 定义、会话、任务等）
- **导出格式**: SQL 文本文件
- **恢复方式**: `psql` 命令导入

### Redis

- **用途**: 热缓存（24小时 TTL）
- **导出格式**: RDB 二进制文件
- **恢复方式**: 替换 dump.rdb 文件后重启

### Qdrant

- **用途**: 向量数据库（语义搜索）
- **导出格式**: tar.gz 压缩包
- **恢复方式**: 解压到数据卷

### Neo4j

- **用途**: 图数据库（实体关系）
- **导出格式**: tar.gz 压缩包 或 neo4j-admin dump
- **恢复方式**: 解压或 neo4j-admin load

### MinIO

- **用途**: 对象存储（原始文件归档）
- **导出格式**: tar.gz 压缩包
- **恢复方式**: 解压到数据卷

## 备份策略建议

### 开发环境

```bash
# 每天自动备份（添加到 crontab）
0 2 * * * cd /path/to/playground && ./data-migrate.sh backup

# 保留最近 7 天的备份
0 3 * * * find /path/to/playground/backups -name "*.tar.gz" -mtime +7 -delete
```

### 生产环境

```bash
# 每小时备份
0 * * * * cd /path/to/playground && ./data-migrate.sh backup

# 保留最近 30 天的备份
0 1 * * * find /path/to/playground/backups -name "*.tar.gz" -mtime +30 -delete

# 同步到远程存储（AWS S3 等）
0 4 * * * aws s3 sync /path/to/playground/backups/ s3://my-backup-bucket/playground/
```

## 故障排查

### 导出失败

```bash
# 检查服务是否运行
./manage.sh services status

# 检查 Docker
docker ps | grep playground

# 手动导出 PostgreSQL
docker exec playground-postgres-1 pg_dump -U postgres agent_platform > backup.sql
```

### 导入失败

```bash
# 确保服务已启动
./manage.sh services start

# 检查日志
./manage.sh services logs postgres

# 检查导入目录
ls -la ./data-export/20240115_120000/
```

### 内存不足

```bash
# 减小 Redis 内存限制
vim .env
# REDIS_MEMORY_LIMIT=128m

# 重启服务
./manage.sh services restart
```

## 相关命令

```bash
# 管理数据
./data-migrate.sh export      # 导出
./data-migrate.sh import      # 导入
./data-migrate.sh backup      # 备份
./data-migrate.sh restore     # 恢复
./data-migrate.sh list        # 列表
./data-migrate.sh clean       # 清理

# 管理服务
./manage.sh services start    # 启动基础设施
./manage.sh services stop     # 停止基础设施
./manage.sh services status   # 查看状态
./manage.sh services logs     # 查看日志
```

## 帮助

```bash
./data-migrate.sh help
```
