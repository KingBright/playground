#!/bin/bash

# Agent Playground - 一站式构建脚本
# 用法: ./build.sh [dev|release|clean|help]

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_DIR="$PROJECT_ROOT/web"
DIST_DIR="$PROJECT_ROOT/dist"
STATIC_DIR="$PROJECT_ROOT/crates/api/static"

# 打印帮助信息
print_help() {
    echo -e "${BLUE}Agent Playground 构建脚本${NC}"
    echo ""
    echo "用法: ./build.sh [命令]"
    echo ""
    echo "命令:"
    echo "  dev       - 开发模式构建 (快速构建，包含调试信息)"
    echo "  release   - 生产模式构建 (优化构建，用于部署)"
    echo "  frontend  - 仅构建前端"
    echo "  backend   - 仅构建后端"
    echo "  clean     - 清理所有构建产物"
    echo "  run       - 构建并运行服务"
    echo "  watch     - 开发模式监视文件变化并自动重建"
    echo "  help      - 显示此帮助信息"
    echo ""
    echo "环境变量:"
    echo "  API_PORT  - API服务器端口 (默认: 8080)"
    echo "  RUST_LOG  - 日志级别 (默认: info)"
}

# 打印带颜色的消息
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."

    # 检查 Node.js
    if ! command -v node &> /dev/null; then
        log_error "Node.js 未安装，请先安装 Node.js"
        exit 1
    fi

    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo 未安装，请先安装 Rust"
        exit 1
    fi

    log_success "依赖检查通过"
}

# 构建前端
build_frontend() {
    log_info "构建前端..."

    cd "$WEB_DIR"

    # 安装依赖
    if [ ! -d "node_modules" ]; then
        log_info "安装前端依赖..."
        npm install
    fi

    # 构建
    npm run build

    log_success "前端构建完成"
}

# 复制静态文件到 Rust 目录
copy_static_files() {
    log_info "复制静态文件到 Rust 目录..."

    # 创建静态文件目录
    mkdir -p "$STATIC_DIR"

    # 复制构建产物
    if [ -d "$WEB_DIR/dist" ]; then
        cp -r "$WEB_DIR/dist"/* "$STATIC_DIR/"
        log_success "静态文件复制完成: $STATIC_DIR"
    else
        log_error "前端构建产物不存在: $WEB_DIR/dist"
        exit 1
    fi
}

# 构建后端
build_backend() {
    local mode=$1
    log_info "构建后端 (${mode}模式)..."

    cd "$PROJECT_ROOT"

    if [ "$mode" = "release" ]; then
        cargo build --release -p api
    else
        cargo build -p api
    fi

    log_success "后端构建完成"
}

# 清理构建产物
clean() {
    log_info "清理构建产物..."

    cd "$PROJECT_ROOT"

    # 清理 Rust 构建
    cargo clean

    # 清理前端构建
    if [ -d "$WEB_DIR/dist" ]; then
        rm -rf "$WEB_DIR/dist"
    fi
    if [ -d "$WEB_DIR/node_modules" ]; then
        rm -rf "$WEB_DIR/node_modules"
    fi

    # 清理静态文件
    if [ -d "$STATIC_DIR" ]; then
        rm -rf "$STATIC_DIR"
    fi

    # 清理 dist 目录
    if [ -d "$DIST_DIR" ]; then
        rm -rf "$DIST_DIR"
    fi

    log_success "清理完成"
}

# 运行服务
run_server() {
    log_info "启动服务..."

    cd "$PROJECT_ROOT"

    # 设置默认环境变量
    export API_PORT=${API_PORT:-8080}
    export RUST_LOG=${RUST_LOG:-info}

    # 运行
    cargo run -p api
}

# 开发模式监视
watch_mode() {
    log_info "启动开发监视模式..."

    # 检查是否安装了 cargo-watch
    if ! cargo watch --version &> /dev/null; then
        log_warning "cargo-watch 未安装，正在安装..."
        cargo install cargo-watch
    fi

    # 检查是否安装了 concurrently
    if ! command -v concurrently &> /dev/null; then
        log_warning "concurrently 未安装，尝试全局安装..."
        npm install -g concurrently 2>/dev/null || {
            log_error "无法安装 concurrently，请手动安装: npm install -g concurrently"
            exit 1
        }
    fi

    cd "$PROJECT_ROOT"

    # 设置默认环境变量
    export API_PORT=${API_PORT:-8080}
    export RUST_LOG=${RUST_LOG:-debug}

    log_info "启动前端开发服务器和 Rust 热重载..."

    # 并行运行前端 dev server 和 Rust watch
    concurrently \
        --names "frontend,backend" \
        --prefix-colors "cyan,yellow" \
        "cd $WEB_DIR && npm run dev" \
        "cargo watch -x 'run -p api'"
}

# 主函数
main() {
    local command=${1:-dev}

    case "$command" in
        dev)
            check_dependencies
            build_frontend
            copy_static_files
            build_backend debug
            log_success "开发模式构建完成！"
            log_info "运行 ./build.sh run 启动服务"
            ;;
        release)
            check_dependencies
            build_frontend
            copy_static_files
            build_backend release
            log_success "生产模式构建完成！"
            log_info "二进制文件位于: target/release/api"
            ;;
        frontend)
            check_dependencies
            build_frontend
            ;;
        backend)
            check_dependencies
            build_backend debug
            ;;
        clean)
            clean
            ;;
        run)
            run_server
            ;;
        watch)
            check_dependencies
            watch_mode
            ;;
        help|--help|-h)
            print_help
            ;;
        *)
            log_error "未知命令: $command"
            print_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"
