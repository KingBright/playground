#!/bin/bash

# Agent Playground - 数据迁移工具
# 支持：导出、导入、备份、清理
#
# 用法: ./data-migrate.sh <command> [options]
#
# 命令:
#   export [path]    导出所有数据到指定目录（默认: ./data-export/YYYYMMDD_HHMMSS/）
#   import <path>    从指定目录导入数据
#   backup [path]    创建压缩备份（默认: ./backups/）
#   list             列出所有备份
#   clean            清理所有数据（警告：会删除所有数据！）
#   restore <path>   恢复数据并重启服务

set -e

# =============================================================================
# 配置
# =============================================================================
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_EXPORT_DIR="$PROJECT_ROOT/data-export"
DEFAULT_BACKUP_DIR="$PROJECT_ROOT/backups"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.yml"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_section() { echo -e "${CYAN}▶ $1${NC}"; }

# 获取 docker compose 命令
docker_compose_cmd() {
    if docker compose version &> /dev/null; then
        echo "docker compose"
    else
        echo "docker-compose"
    fi
}

# 检查 Docker
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker 未安装"
        exit 1
    fi
    if ! docker info &> /dev/null; then
        log_error "Docker 守护进程未运行"
        exit 1
    fi
}

# =============================================================================
# 导出数据
# =============================================================================
cmd_export() {
    local export_dir=${1:-"$DEFAULT_EXPORT_DIR/$(date +%Y%m%d_%H%M%S)"}

    log_section "导出数据"
    log_info "目标目录: $export_dir"

    check_docker

    mkdir -p "$export_dir"

    local compose_cmd=$(docker_compose_cmd)

    # 1. 导出 PostgreSQL
    log_info "导出 PostgreSQL..."
    if docker ps | grep -q "playground-postgres-1\|postgres"; then
        docker exec playground-postgres-1 pg_dump -U postgres agent_platform > "$export_dir/postgres.sql" 2>/dev/null || {
            log_warning "PostgreSQL 导出失败（可能服务未运行）"
            touch "$export_dir/postgres.sql.failed"
        }
        if [ -f "$export_dir/postgres.sql" ] && [ -s "$export_dir/postgres.sql" ]; then
            log_success "PostgreSQL 导出完成"
        fi
    else
        # 如果容器未运行，尝试直接从 volume 复制（如果存在）
        log_warning "PostgreSQL 容器未运行，尝试从 volume 复制..."
        local pg_volume=$(docker volume ls -q | grep -i postgres | head -1)
        if [ -n "$pg_volume" ]; then
            log_info "发现 volume: $pg_volume"
            log_warning "请手动启动 PostgreSQL 后重新导出"
        fi
    fi

    # 2. 导出 Redis
    log_info "导出 Redis..."
    if docker ps | grep -q "playground-redis-1\|redis"; then
        # 先执行 SAVE 确保数据写入磁盘
        docker exec playground-redis-1 redis-cli SAVE 2>/dev/null || true
        sleep 1
        # 复制 RDB 文件
        docker cp playground-redis-1:/data/dump.rdb "$export_dir/redis.rdb" 2>/dev/null || {
            log_warning "Redis 导出失败"
            touch "$export_dir/redis.rdb.failed"
        }
        if [ -f "$export_dir/redis.rdb" ]; then
            log_success "Redis 导出完成"
        fi
    else
        log_warning "Redis 容器未运行"
    fi

    # 3. 导出 Qdrant
    log_info "导出 Qdrant..."
    if docker ps | grep -q "playground-qdrant-1\|qdrant"; then
        # Qdrant 数据存储在 volume 中，我们需要创建一个快照
        local qdrant_volume=$(docker volume ls -q | grep -i qdrant | head -1)
        if [ -n "$qdrant_volume" ]; then
            docker run --rm -v "$qdrant_volume:/source" -v "$export_dir:/backup" alpine \
                tar czf /backup/qdrant.tar.gz -C /source . 2>/dev/null || {
                log_warning "Qdrant 导出失败"
                touch "$export_dir/qdrant.tar.gz.failed"
            }
            if [ -f "$export_dir/qdrant.tar.gz" ]; then
                log_success "Qdrant 导出完成"
            fi
        fi
    else
        log_warning "Qdrant 容器未运行"
    fi

    # 4. 导出 Neo4j
    log_info "导出 Neo4j..."
    if docker ps | grep -q "playground-neo4j-1\|neo4j"; then
        # Neo4j 需要创建一个 dump
        docker exec playground-neo4j-1 neo4j-admin database dump neo4j --to-path=/tmp/neo4j.dump 2>/dev/null || {
            log_warning "Neo4j dump 失败，尝试复制数据文件..."
        }

        if docker exec playground-neo4j-1 test -f /tmp/neo4j.dump; then
            docker cp playground-neo4j-1:/tmp/neo4j.dump "$export_dir/neo4j.dump"
            log_success "Neo4j 导出完成"
        else
            # 直接复制数据目录
            local neo4j_volume=$(docker volume ls -q | grep -i neo4j | grep -v logs | head -1)
            if [ -n "$neo4j_volume" ]; then
                docker run --rm -v "$neo4j_volume:/source" -v "$export_dir:/backup" alpine \
                    tar czf /backup/neo4j.tar.gz -C /source . 2>/dev/null || {
                    log_warning "Neo4j 导出失败"
                    touch "$export_dir/neo4j.tar.gz.failed"
                }
                if [ -f "$export_dir/neo4j.tar.gz" ]; then
                    log_success "Neo4j 数据文件导出完成"
                fi
            fi
        fi
    else
        log_warning "Neo4j 容器未运行"
    fi

    # 5. 导出 MinIO
    log_info "导出 MinIO..."
    if docker ps | grep -q "playground-minio-1\|minio"; then
        # MinIO 使用 mc 客户端导出
        local minio_volume=$(docker volume ls -q | grep -i minio | head -1)
        if [ -n "$minio_volume" ]; then
            docker run --rm -v "$minio_volume:/source" -v "$export_dir:/backup" alpine \
                tar czf /backup/minio.tar.gz -C /source . 2>/dev/null || {
                log_warning "MinIO 导出失败"
                touch "$export_dir/minio.tar.gz.failed"
            }
            if [ -f "$export_dir/minio.tar.gz" ]; then
                log_success "MinIO 导出完成"
            fi
        fi
    else
        log_warning "MinIO 容器未运行"
    fi

    # 6. 创建导出信息文件
    cat > "$export_dir/export-info.json" << EOF
{
    "export_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "version": "1.0",
    "hostname": "$(hostname)",
    "files": {
        "postgres": $(test -f "$export_dir/postgres.sql" && echo '"postgres.sql"' || echo 'null'),
        "redis": $(test -f "$export_dir/redis.rdb" && echo '"redis.rdb"' || echo 'null'),
        "qdrant": $(test -f "$export_dir/qdrant.tar.gz" && echo '"qdrant.tar.gz"' || echo 'null'),
        "neo4j": $(test -f "$export_dir/neo4j.dump" && echo '"neo4j.dump"' || test -f "$export_dir/neo4j.tar.gz" && echo '"neo4j.tar.gz"' || echo 'null'),
        "minio": $(test -f "$export_dir/minio.tar.gz" && echo '"minio.tar.gz"' || echo 'null')
    }
}
EOF

    log_success "数据导出完成: $export_dir"
    log_info "导出内容:"
    ls -lh "$export_dir"
}

# =============================================================================
# 导入数据
# =============================================================================
cmd_import() {
    local import_dir=${1:-}

    if [ -z "$import_dir" ]; then
        log_error "请指定导入目录"
        echo "用法: ./data-migrate.sh import <path>"
        exit 1
    fi

    if [ ! -d "$import_dir" ]; then
        log_error "导入目录不存在: $import_dir"
        exit 1
    fi

    log_section "导入数据"
    log_info "源目录: $import_dir"

    check_docker

    local compose_cmd=$(docker_compose_cmd)

    # 检查服务是否运行
    log_info "检查服务状态..."
    if ! docker ps | grep -q "playground-postgres-1"; then
        log_warning "PostgreSQL 未运行，请先启动服务: ./manage.sh services start"
        exit 1
    fi

    # 1. 导入 PostgreSQL
    if [ -f "$import_dir/postgres.sql" ] && [ -s "$import_dir/postgres.sql" ]; then
        log_info "导入 PostgreSQL..."
        docker exec -i playground-postgres-1 psql -U postgres agent_platform < "$import_dir/postgres.sql"
        log_success "PostgreSQL 导入完成"
    else
        log_warning "未找到 PostgreSQL 备份文件"
    fi

    # 2. 导入 Redis
    if [ -f "$import_dir/redis.rdb" ]; then
        log_info "导入 Redis..."
        # 停止 Redis
        docker stop playground-redis-1
        # 复制 RDB 文件
        docker cp "$import_dir/redis.rdb" playground-redis-1:/data/dump.rdb
        # 启动 Redis
        docker start playground-redis-1
        log_success "Redis 导入完成（服务已重启）"
    else
        log_warning "未找到 Redis 备份文件"
    fi

    # 3. 导入 Qdrant
    if [ -f "$import_dir/qdrant.tar.gz" ]; then
        log_info "导入 Qdrant..."
        docker stop playground-qdrant-1
        local qdrant_volume=$(docker volume ls -q | grep -i qdrant | head -1)
        if [ -n "$qdrant_volume" ]; then
            docker run --rm -v "$qdrant_volume:/target" -v "$import_dir:/backup" alpine \
                sh -c "rm -rf /target/* && tar xzf /backup/qdrant.tar.gz -C /target"
            log_success "Qdrant 导入完成"
        fi
        docker start playground-qdrant-1
    else
        log_warning "未找到 Qdrant 备份文件"
    fi

    # 4. 导入 Neo4j
    if [ -f "$import_dir/neo4j.dump" ]; then
        log_info "导入 Neo4j (使用 dump 文件)..."
        docker stop playground-neo4j-1
        docker exec playground-neo4j-1 neo4j-admin database load neo4j --from-path=/tmp/neo4j.dump --force 2>/dev/null || {
            log_warning "Neo4j dump 恢复失败，尝试直接复制..."
        }
        docker start playground-neo4j-1
        log_success "Neo4j 导入完成"
    elif [ -f "$import_dir/neo4j.tar.gz" ]; then
        log_info "导入 Neo4j (使用 tar 文件)..."
        docker stop playground-neo4j-1
        local neo4j_volume=$(docker volume ls -q | grep -i neo4j | grep -v logs | head -1)
        if [ -n "$neo4j_volume" ]; then
            docker run --rm -v "$neo4j_volume:/target" -v "$import_dir:/backup" alpine \
                sh -c "rm -rf /target/* && tar xzf /backup/neo4j.tar.gz -C /target"
        fi
        docker start playground-neo4j-1
        log_success "Neo4j 导入完成"
    else
        log_warning "未找到 Neo4j 备份文件"
    fi

    # 5. 导入 MinIO
    if [ -f "$import_dir/minio.tar.gz" ]; then
        log_info "导入 MinIO..."
        docker stop playground-minio-1
        local minio_volume=$(docker volume ls -q | grep -i minio | head -1)
        if [ -n "$minio_volume" ]; then
            docker run --rm -v "$minio_volume:/target" -v "$import_dir:/backup" alpine \
                sh -c "rm -rf /target/* && tar xzf /backup/minio.tar.gz -C /target"
        fi
        docker start playground-minio-1
        log_success "MinIO 导入完成"
    else
        log_warning "未找到 MinIO 备份文件"
    fi

    log_success "数据导入完成！"
    log_info "请检查服务状态: ./manage.sh services status"
}

# =============================================================================
# 创建压缩备份
# =============================================================================
cmd_backup() {
    local backup_dir=${1:-"$DEFAULT_BACKUP_DIR"}
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local temp_export="$DEFAULT_EXPORT_DIR/$timestamp"

    log_section "创建压缩备份"
    log_info "备份目录: $backup_dir"

    mkdir -p "$backup_dir"

    # 先导出到临时目录
    cmd_export "$temp_export"

    # 创建压缩包
    local backup_file="$backup_dir/agent-playground-backup-$timestamp.tar.gz"
    log_info "创建压缩包: $backup_file"

    tar czf "$backup_file" -C "$temp_export" .

    # 清理临时目录
    rm -rf "$temp_export"

    log_success "备份完成: $backup_file"
    log_info "文件大小: $(ls -lh "$backup_file" | awk '{print $5}')"
}

# =============================================================================
# 列出备份
# =============================================================================
cmd_list() {
    log_section "备份列表"

    if [ -d "$DEFAULT_BACKUP_DIR" ]; then
        log_info "备份目录: $DEFAULT_BACKUP_DIR"
        ls -lh "$DEFAULT_BACKUP_DIR" | grep -E "\.tar\.gz$|\.zip$" || log_warning "没有找到备份文件"
    else
        log_warning "备份目录不存在: $DEFAULT_BACKUP_DIR"
    fi

    if [ -d "$DEFAULT_EXPORT_DIR" ]; then
        echo ""
        log_info "导出目录: $DEFAULT_EXPORT_DIR"
        ls -ld "$DEFAULT_EXPORT_DIR"/*/ 2>/dev/null | tail -10 || log_warning "没有找到导出目录"
    fi
}

# =============================================================================
# 清理数据
# =============================================================================
cmd_clean() {
    log_section "清理数据"

    log_warning "⚠️  这将删除所有数据，包括："
    echo "  - PostgreSQL 数据库"
    echo "  - Redis 缓存"
    echo "  - Qdrant 向量数据"
    echo "  - Neo4j 图数据"
    echo "  - MinIO 对象存储"
    echo ""

    read -p "确定要继续吗? 输入 'yes' 确认: " -r
    echo

    if [[ ! $REPLY =~ ^yes$ ]]; then
        log_info "取消操作"
        exit 0
    fi

    check_docker

    local compose_cmd=$(docker_compose_cmd)

    log_info "停止服务..."
    $compose_cmd -f "$COMPOSE_FILE" down 2>/dev/null || true

    log_info "删除数据卷..."
    docker volume rm playground_redis-data playground_postgres-data playground_qdrant-data playground_neo4j-data playground_minio-data 2>/dev/null || true

    log_success "数据已清理"
    log_info "可以重新启动服务: ./manage.sh services start"
}

# =============================================================================
# 恢复并重启
# =============================================================================
cmd_restore() {
    local import_dir=${1:-}

    if [ -z "$import_dir" ]; then
        log_error "请指定恢复目录或备份文件"
        echo "用法: ./data-migrate.sh restore <path>"
        exit 1
    fi

    # 如果是压缩包，先解压
    if [[ "$import_dir" == *.tar.gz ]]; then
        if [ ! -f "$import_dir" ]; then
            log_error "备份文件不存在: $import_dir"
            exit 1
        fi

        local temp_dir="$DEFAULT_EXPORT_DIR/restore-$(date +%Y%m%d_%H%M%S)"
        log_info "解压备份文件到: $temp_dir"
        mkdir -p "$temp_dir"
        tar xzf "$import_dir" -C "$temp_dir"
        import_dir="$temp_dir"
    fi

    # 检查服务是否运行
    check_docker

    log_info "停止服务..."
    local compose_cmd=$(docker_compose_cmd)
    $compose_cmd -f "$COMPOSE_FILE" stop 2>/dev/null || true

    # 导入数据
    cmd_import "$import_dir"

    log_success "恢复完成！"
    log_info "服务已重启，请检查状态: ./manage.sh services status"
}

# =============================================================================
# 帮助
# =============================================================================
cmd_help() {
    cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════════════╗
║              Agent Playground - 数据迁移工具                               ║
╚═══════════════════════════════════════════════════════════════════════════╝

用法: ./data-migrate.sh <command> [options]

命令:
  export [path]     导出所有数据到指定目录
                    默认: ./data-export/YYYYMMDD_HHMMSS/

  import <path>     从指定目录导入数据

  backup [path]     创建压缩备份（默认: ./backups/）

  list              列出所有备份

  clean             清理所有数据（警告：会删除所有数据！）

  restore <path>    恢复数据并重启服务
                    支持目录或 .tar.gz 文件

示例:
  # 导出数据
  ./data-migrate.sh export
  ./data-migrate.sh export ./my-backup

  # 创建压缩备份
  ./data-migrate.sh backup

  # 查看备份列表
  ./data-migrate.sh list

  # 从目录恢复
  ./data-migrate.sh restore ./data-export/20240115_120000

  # 从压缩包恢复
  ./data-migrate.sh restore ./backups/agent-playground-backup-20240115.tar.gz

  # 迁移到新机器
  # 1. 在旧机器导出
  ./data-migrate.sh backup
  # 2. 复制 backups/ 目录到新机器
  # 3. 在新机器恢复
  ./data-migrate.sh restore ./backups/agent-playground-backup-xxx.tar.gz

数据包括:
  - PostgreSQL: SQL 导出文件
  - Redis: RDB 持久化文件
  - Qdrant: 向量数据压缩包
  - Neo4j: 图数据压缩包
  - MinIO: 对象存储压缩包

EOF
}

# =============================================================================
# 主函数
# =============================================================================
main() {
    local command=${1:-help}
    shift || true

    case "$command" in
        export)
            cmd_export "$@"
            ;;
        import)
            cmd_import "$@"
            ;;
        backup)
            cmd_backup "$@"
            ;;
        list)
            cmd_list
            ;;
        clean)
            cmd_clean
            ;;
        restore)
            cmd_restore "$@"
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
