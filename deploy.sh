#!/bin/bash

# Agent Playground - 一键部署脚本
# 用法: ./deploy.sh [command] [options]
#
# 命令:
#   prod      - 生产模式启动（Docker 完整部署）
#   up        - 启动 Docker 服务
#   down      - 停止 Docker 服务
#   restart   - 重启 Docker 服务
#   status    - 查看服务状态
#   logs      - 查看日志
#   build     - 构建镜像
#   clean     - 清理数据卷
#   update    - 更新并重新部署
#   backup    - 备份数据
#   help      - 显示帮助
#
# 注意: 此脚本专注于 Docker 部署
#       本地开发请使用: ./manage.sh dev

set -e

# =============================================================================
# 颜色定义
# =============================================================================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# =============================================================================
# 配置
# =============================================================================
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.yml"
ENV_FILE="$PROJECT_ROOT/.env"

# =============================================================================
# 日志函数
# =============================================================================
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_section() { echo -e "${MAGENTA}▶ $1${NC}"; }

# =============================================================================
# 检查 Docker
# =============================================================================
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker 未安装，请先安装 Docker"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose 未安装，请先安装 Docker Compose"
        exit 1
    fi

    # 检查 Docker 是否运行
    if ! docker info &> /dev/null; then
        log_error "Docker 守护进程未运行，请先启动 Docker"
        exit 1
    fi

    log_success "Docker 环境检查通过"
}

# 获取 docker compose 命令
docker_compose_cmd() {
    if docker compose version &> /dev/null; then
        echo "docker compose"
    else
        echo "docker-compose"
    fi
}

# 检测系统架构
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "amd64"
            ;;
        arm64|aarch64)
            echo "arm64"
            ;;
        *)
            echo "$arch"
            ;;
    esac
}

# 获取 Docker 平台信息
docker_platform() {
    if command -v docker &> /dev/null; then
        docker version --format '{{.Server.Os}}/{{.Server.Arch}}' 2>/dev/null || echo "unknown"
    else
        echo "unknown"
    fi
}

# 显示架构信息
show_arch_info() {
    local host_arch=$(detect_arch)
    local docker_info=$(docker_platform)

    log_info "系统架构: $host_arch"
    log_info "Docker 平台: $docker_info"

    # 如果是 Apple Silicon，提供提示
    if [[ "$host_arch" == "arm64" ]] && [[ "$OSTYPE" == "darwin"* ]]; then
        log_info "检测到 Apple Silicon (M1/M2/M3)"
        log_info "将使用 ARM64 架构镜像以获得最佳性能"
    fi
}

# =============================================================================
# 环境配置
# =============================================================================
setup_env() {
    if [ ! -f "$ENV_FILE" ]; then
        log_warning "未找到 .env 文件，正在创建默认配置..."
        cat > "$ENV_FILE" << 'EOF'
# ==========================================
# Agent Playground - 环境配置
# ==========================================

# 应用配置
API_PORT=8080
WEB_PORT=3000
RUST_LOG=info

# 数据库配置
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=agent_platform

# Redis 配置
REDIS_PORT=6379

# Qdrant (向量数据库) 配置
QDRANT_PORT=6333

# Neo4j (图数据库) 配置
NEO4J_HTTP_PORT=7474
NEO4J_BOLT_PORT=7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password

# MinIO (对象存储) 配置
MINIO_PORT=9000
MINIO_CONSOLE_PORT=9001
MINIO_ROOT_USER=minioadmin
MINIO_ROOT_PASSWORD=minioadmin

# 内部服务 URL (Docker 网络内使用)
REDIS_URL=redis://redis:6379
DATABASE_URL=postgres://postgres:postgres@postgres:5432/agent_platform
QDRANT_URL=http://qdrant:6333
NEO4J_URL=bolt://neo4j:7687
EOF
        log_success "已创建默认 .env 文件"
        log_info "你可以编辑 .env 文件来自定义配置"
    fi
}

# =============================================================================
# 启动服务
# =============================================================================
cmd_up() {
    check_docker
    setup_env
    show_arch_info

    # 如果是 Apple Silicon，设置 ARM64 平台
    local host_arch=$(detect_arch)
    if [[ "$host_arch" == "arm64" ]] && [[ "$OSTYPE" == "darwin"* ]]; then
        export DOCKER_PLATFORM="linux/arm64"
    fi

    log_section "启动 Agent Playground 服务"

    local profile=${1:-}
    local compose_cmd=$(docker_compose_cmd)

    if [ "$profile" = "web" ]; then
        log_info "启动模式: 包含独立前端服务"
        $compose_cmd -f "$COMPOSE_FILE" --profile with-web up -d
    elif [ "$profile" = "init" ]; then
        log_info "启动模式: 包含初始化服务"
        $compose_cmd -f "$COMPOSE_FILE" --profile init up -d
    else
        log_info "启动模式: 基础服务 (API + 数据库)"
        $compose_cmd -f "$COMPOSE_FILE" up -d
    fi

    log_success "服务启动完成"
    show_urls
}

# =============================================================================
# 停止服务
# =============================================================================
cmd_down() {
    log_section "停止 Agent Playground 服务"

    local compose_cmd=$(docker_compose_cmd)
    $compose_cmd -f "$COMPOSE_FILE" down

    log_success "服务已停止"
}

# =============================================================================
# 完全清理（包括数据卷）
# =============================================================================
cmd_clean() {
    log_section "清理所有服务和数据"

    read -p "这将删除所有数据卷，确定要继续吗? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "取消操作"
        exit 0
    fi

    local compose_cmd=$(docker_compose_cmd)
    $compose_cmd -f "$COMPOSE_FILE" down -v --remove-orphans

    # 清理构建缓存
    docker system prune -f

    log_success "清理完成"
}

# =============================================================================
# 重启服务
# =============================================================================
cmd_restart() {
    log_section "重启 Agent Playground 服务"
    cmd_down
    sleep 2
    cmd_up "$@"
}

# =============================================================================
# 查看状态
# =============================================================================
cmd_status() {
    log_section "服务状态"

    local compose_cmd=$(docker_compose_cmd)
    $compose_cmd -f "$COMPOSE_FILE" ps

    echo ""
    log_section "资源使用"
    $compose_cmd -f "$COMPOSE_FILE" top 2>/dev/null || echo "无法获取资源信息"
}

# =============================================================================
# 查看日志
# =============================================================================
cmd_logs() {
    local service=${1:-}
    local compose_cmd=$(docker_compose_cmd)

    if [ -n "$service" ]; then
        $compose_cmd -f "$COMPOSE_FILE" logs -f "$service"
    else
        $compose_cmd -f "$COMPOSE_FILE" logs -f
    fi
}

# =============================================================================
# 构建镜像
# =============================================================================
cmd_build() {
    log_section "构建 Docker 镜像"

    local compose_cmd=$(docker_compose_cmd)
    $compose_cmd -f "$COMPOSE_FILE" build --no-cache

    log_success "镜像构建完成"
}

# =============================================================================
# 更新部署
# =============================================================================
cmd_update() {
    log_section "更新 Agent Playground"

    # 拉取最新代码（如果在 git 仓库中）
    if [ -d "$PROJECT_ROOT/.git" ]; then
        log_info "拉取最新代码..."
        cd "$PROJECT_ROOT"
        git pull origin main || log_warning "拉取代码失败，继续本地构建"
    fi

    # 重新构建
    cmd_build

    # 重启服务
    cmd_restart

    log_success "更新完成"
}

# =============================================================================
# 开发模式
# =============================================================================
cmd_dev() {
    log_section "启动开发模式"
    check_docker
    setup_env

    # 只启动依赖服务（数据库等）
    local compose_cmd=$(docker_compose_cmd)

    log_info "启动基础设施服务..."
    $compose_cmd -f "$COMPOSE_FILE" up -d redis postgres qdrant neo4j minio

    log_info "等待服务就绪..."
    sleep 5

    # 检查服务健康状态
    log_info "检查服务健康状态..."
    local max_attempts=30
    local attempt=1

    while [ $attempt -le $max_attempts ]; do
        if docker inspect --format='{{.State.Health.Status}}' playground-redis-1 2>/dev/null | grep -q "healthy"; then
            break
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    echo ""

    show_urls
    log_success "基础设施就绪！"
    log_info "你现在可以本地运行 API 和前端开发服务器："
    echo "  ./manage.sh run        # 运行 API 服务"
    echo "  ./manage.sh dev        # 运行开发模式（热重载）"
    echo "  ./manage.sh services   # 管理基础设施服务"
}

# =============================================================================
# 生产模式
# =============================================================================
cmd_prod() {
    log_section "启动生产模式"
    check_docker
    setup_env
    show_arch_info

    # 如果是 Apple Silicon，设置 ARM64 平台
    local host_arch=$(detect_arch)
    if [[ "$host_arch" == "arm64" ]] && [[ "$OSTYPE" == "darwin"* ]]; then
        export DOCKER_PLATFORM="linux/arm64"
        log_info "将为 Apple Silicon 构建 ARM64 镜像"
    fi

    # 构建优化镜像
    log_info "构建优化镜像..."
    cmd_build

    # 启动所有服务
    cmd_up

    # 运行初始化
    log_info "运行初始化..."
    local compose_cmd=$(docker_compose_cmd)
    $compose_cmd -f "$COMPOSE_FILE" --profile init run --rm init

    log_success "生产环境部署完成！"
    show_urls
}

# =============================================================================
# 显示访问地址
# =============================================================================
show_urls() {
    echo ""
    log_section "服务访问地址"
    echo ""
    echo -e "${CYAN}API 服务:${NC}      http://localhost:${API_PORT:-8080}"
    echo -e "${CYAN}API 文档:${NC}      http://localhost:${API_PORT:-8080}/api/docs"
    echo -e "${CYAN}健康检查:${NC}     http://localhost:${API_PORT:-8080}/api/health"
    echo ""
    echo -e "${CYAN}Redis:${NC}         localhost:${REDIS_PORT:-6379}"
    echo -e "${CYAN}PostgreSQL:${NC}    localhost:${POSTGRES_PORT:-5432}"
    echo -e "${CYAN}Qdrant:${NC}        http://localhost:${QDRANT_PORT:-6333}"
    echo -e "${CYAN}Neo4j Browser:${NC} http://localhost:${NEO4J_HTTP_PORT:-7474}"
    echo -e "${CYAN}MinIO Console:${NC} http://localhost:${MINIO_CONSOLE_PORT:-9001}"
    echo ""
    echo -e "${YELLOW}默认凭据:${NC}"
    echo "  PostgreSQL: postgres/postgres"
    echo "  Neo4j:      neo4j/password"
    echo "  MinIO:      minioadmin/minioadmin"
    echo ""
}

# =============================================================================
# 数据库迁移（预留）
# =============================================================================
cmd_migrate() {
    log_section "运行数据库迁移"
    log_warning "数据库迁移功能尚未实现"
    # 这里可以添加 sqlx migrate run 等命令
}

# =============================================================================
# 备份数据
# =============================================================================
cmd_backup() {
    log_section "备份数据"

    local backup_dir="$PROJECT_ROOT/backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$backup_dir"

    log_info "备份到: $backup_dir"

    # 备份 PostgreSQL
    docker exec playground-postgres-1 pg_dump -U postgres agent_platform > "$backup_dir/postgres.sql" 2>/dev/null || log_warning "PostgreSQL 备份失败"

    # 备份 Redis
    docker exec playground-redis-1 redis-cli SAVE 2>/dev/null || log_warning "Redis 保存失败"
    docker cp playground-redis-1:/data/dump.rdb "$backup_dir/redis.rdb" 2>/dev/null || log_warning "Redis 备份失败"

    log_success "备份完成: $backup_dir"
}

# =============================================================================
# 帮助
# =============================================================================
cmd_help() {
    cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════════════╗
║              Agent Playground - 一键部署脚本                               ║
╚═══════════════════════════════════════════════════════════════════════════╝

用法: ./deploy.sh <command> [options]

命令:
  prod              生产模式（Docker 完整部署，推荐）

  up [web|init]     启动 Docker 服务
                    web  - 包含独立前端服务
                    init - 包含初始化服务

  down              停止所有服务

  restart [web]     重启所有服务

  status            查看服务状态

  logs [service]    查看日志
                    service: api, redis, postgres, qdrant, neo4j, minio

  build             构建 Docker 镜像

  clean             清理所有服务和数据卷（警告：会删除数据！）

  update            更新代码并重新部署

  backup            备份所有数据

  help              显示此帮助

示例:
  ./deploy.sh prod              # 完整生产部署
  ./deploy.sh up                # 启动基础服务
  ./deploy.sh up web            # 启动包含前端的服务
  ./deploy.sh logs api          # 查看 API 日志
  ./deploy.sh restart           # 重启服务

快速开始:
  1. 生产部署:    ./deploy.sh prod
  2. 查看状态:    ./deploy.sh status
  3. 查看日志:    ./deploy.sh logs

注意:
  本地开发请使用 ./manage.sh:
    ./manage.sh dev          # 开发模式（本地热重载）
    ./manage.sh services     # 管理基础设施服务

EOF
}

# =============================================================================
# 主函数
# =============================================================================
main() {
    local command=${1:-help}
    shift || true

    case "$command" in
        up)
            cmd_up "$@"
            ;;
        down)
            cmd_down
            ;;
        restart)
            cmd_restart "$@"
            ;;
        status)
            cmd_status
            ;;
        logs)
            cmd_logs "$@"
            ;;
        build)
            cmd_build
            ;;
        clean)
            cmd_clean
            ;;
        update)
            cmd_update
            ;;
        prod)
            cmd_prod
            ;;
        backup)
            cmd_backup
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            log_error "未知命令: $command"
            echo ""
            cmd_help
            exit 1
            ;;
    esac
}

main "$@"
