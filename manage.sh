#!/bin/bash

# Agent Playground - 项目管理脚本
# 用法: ./manage.sh <command> [subcommand] [options]
#
# 命令列表:
#   dev       - 开发模式（本地启动所有服务，热重载）
#   build     - 构建项目 (dev, release, frontend, backend)
#   run       - 单独运行 API 服务
#   test      - 运行测试 (rust, frontend, all, coverage)
#   clean     - 清理所有构建产物
#   install   - 安装所有依赖
#   lint      - 代码检查 (rust, frontend, all)
#   fmt       - 代码格式化 (rust, frontend, all)
#   check     - 完整检查（lint + test + build）
#   help      - 显示帮助信息
#
# 数据迁移:
#   ./data-migrate.sh backup    - 创建数据备份
#   ./data-migrate.sh restore   - 恢复数据
#   ./data-migrate.sh export    - 导出数据
#

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
NC='\033[0m' # No Color

# =============================================================================
# 项目目录
# =============================================================================
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_DIR="$PROJECT_ROOT/web"
DIST_DIR="$PROJECT_ROOT/dist"
STATIC_DIR="$PROJECT_ROOT/crates/api/static"
CRATES_DIR="$PROJECT_ROOT/crates"

# =============================================================================
# 日志函数
# =============================================================================
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_debug() { echo -e "${CYAN}[DEBUG]${NC} $1"; }
log_section() { echo -e "${MAGENTA}▶ $1${NC}"; }

# =============================================================================
# 加载环境变量
# =============================================================================
load_env() {
    if [ -f "$PROJECT_ROOT/.env" ]; then
        log_debug "加载环境变量: .env"
        set -a
        source "$PROJECT_ROOT/.env"
        set +a
    fi
}

# =============================================================================
# 端口工具函数
# =============================================================================

# 检查端口是否可用
is_port_available() {
    local port=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        ! lsof -Pi :"$port" -sTCP:LISTEN -t >/dev/null 2>&1
    else
        # Linux
        ! ss -tuln | grep -q ":$port "
    fi
}

# 获取随机可用端口
get_available_port() {
    local start_port=${1:-10000}
    local end_port=${2:-65535}
    local port

    # 首先尝试默认端口
    if [ -n "$API_PORT" ] && is_port_available "$API_PORT"; then
        echo "$API_PORT"
        return 0
    fi

    # 尝试 8080-8090 范围
    for port in {8080..8090}; do
        if is_port_available "$port"; then
            echo "$port"
            return 0
        fi
    done

    # 随机端口
    while true; do
        port=$((RANDOM % (end_port - start_port + 1) + start_port))
        if is_port_available "$port"; then
            echo "$port"
            return 0
        fi
    done
}

# =============================================================================
# 检查依赖
# =============================================================================
check_dependencies() {
    log_section "检查依赖"

    local missing_deps=()

    # 检查 Node.js
    if ! command -v node &> /dev/null; then
        missing_deps+=("Node.js")
    else
        local node_version=$(node --version)
        log_info "Node.js: $node_version"
    fi

    # 检查 npm
    if ! command -v npm &> /dev/null; then
        missing_deps+=("npm")
    else
        local npm_version=$(npm --version)
        log_info "npm: $npm_version"
    fi

    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("Rust/Cargo")
    else
        local rust_version=$(rustc --version)
        local cargo_version=$(cargo --version)
        log_info "$rust_version"
        log_info "$cargo_version"
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "缺少以下依赖:"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        exit 1
    fi

    log_success "依赖检查通过"
}

# =============================================================================
# 开发模式 - 本地启动所有服务（热重载）
# =============================================================================
cmd_dev() {
    log_section "启动开发模式（本地进程）"

    check_dependencies
    load_env

    # 检查是否安装了 concurrently
    if ! command -v concurrently &> /dev/null; then
        log_warning "concurrently 未安装，正在安装..."
        npm install -g concurrently 2>/dev/null || {
            log_error "无法安装 concurrently，请手动安装: npm install -g concurrently"
            exit 1
        }
    fi

    # 检查是否安装了 cargo-watch
    if ! cargo watch --version &> /dev/null; then
        log_warning "cargo-watch 未安装，正在安装..."
        cargo install cargo-watch
    fi

    # 清理可能残留的 cargo watch 进程
    if pgrep -f "cargo watch" > /dev/null 2>&1; then
        log_warning "检测到已有的 cargo watch 进程，正在清理..."
        pkill -f "cargo watch" 2>/dev/null || true
        sleep 2
    fi

    log_section "启动应用服务（热重载）"

    cd "$PROJECT_ROOT"

    # 自动检测可用端口
    local detected_port
    detected_port=$(get_available_port)
    export API_PORT=${API_PORT:-$detected_port}
    export RUST_LOG=${RUST_LOG:-debug}
    export NODE_ENV=development
    # 开发模式：使用空目录作为 static，确保 API 显示开发模式日志
    export STATIC_DIR="/tmp/agent-playground-dev-static"
    mkdir -p "$STATIC_DIR"

    log_info "API 端口: $API_PORT"
    log_info "日志级别: $RUST_LOG"
    log_info "Static 目录: $STATIC_DIR (开发模式使用空目录)"

    # 如果设置了 CLEAN_BUILD，先清理构建缓存
    if [ "${CLEAN_BUILD:-}" = "1" ] || [ "${CLEAN_BUILD:-}" = "true" ]; then
        log_warning "CLEAN_BUILD 已启用，正在清理构建缓存..."
        cargo clean -p api 2>/dev/null || cargo clean
        log_success "构建缓存已清理"
    fi

    # 设置前端代理配置
    export API_URL="http://localhost:${API_PORT}"

    # 显示访问地址
    echo ""
    log_section "服务访问地址"
    echo -e "${CYAN}API 服务:${NC}     http://localhost:${API_PORT}"
    echo -e "${CYAN}前端服务:${NC}    http://localhost:5173 (如被占用会自动切换)"
    echo -e "${CYAN}API 文档:${NC}    http://localhost:${API_PORT}/api/docs"
    echo ""

    # 使用 concurrently 启动应用服务
    log_info "正在启动 API 和前端服务..."
    log_info "提示: 按 Ctrl+C 停止 API 和前端"

    # 使用 cargo watch 监视 crates 目录的变化
    # -w: 明确指定监视目录
    # --poll: 在 macOS 上使用轮询模式（更可靠）
    # -x: 执行命令
    local cargo_watch_cmd
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: 使用轮询模式避免 fsevents 问题
        cargo_watch_cmd="cargo watch --poll -w crates -x 'run -p api'"
    else
        # Linux: 使用默认文件系统事件
        cargo_watch_cmd="cargo watch -w crates -x 'run -p api'"
    fi

    concurrently \
        --names "backend,frontend" \
        --prefix-colors "yellow,cyan" \
        --kill-others \
        "$cargo_watch_cmd" \
        "cd $WEB_DIR && API_URL=$API_URL npm run dev" \
        || true

    echo ""
    log_info "API 和前端服务已停止"
}

# =============================================================================
# 构建命令
# =============================================================================
cmd_build() {
    local subcommand=${1:-dev}

    case "$subcommand" in
        dev|debug)
            log_section "开发模式构建"
            check_dependencies
            build_frontend
            copy_static_files
            build_backend debug
            log_success "开发模式构建完成！"
            log_info "运行 './manage.sh run' 启动服务"
            ;;
        release)
            log_section "生产模式构建"
            check_dependencies
            build_frontend
            copy_static_files
            build_backend release
            log_success "生产模式构建完成！"
            log_info "二进制文件位于: target/release/api"
            ;;
        frontend)
            log_section "构建前端"
            check_dependencies
            build_frontend
            ;;
        backend)
            log_section "构建后端"
            check_dependencies
            build_backend debug
            ;;
        *)
            log_error "未知的 build 子命令: $subcommand"
            echo "可用的子命令: dev, release, frontend, backend"
            exit 1
            ;;
    esac
}

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

# =============================================================================
# 运行 API（单独运行）
# =============================================================================
cmd_run() {
    log_section "运行 API 服务"

    cd "$PROJECT_ROOT"

    # 设置默认环境变量
    export API_PORT=${API_PORT:-8080}
    export RUST_LOG=${RUST_LOG:-info}

    log_info "API 端口: $API_PORT"
    log_info "日志级别: $RUST_LOG"

    # 运行
    cargo run -p api
}

# =============================================================================
# 测试命令
# =============================================================================
cmd_test() {
    local subcommand=${1:-all}

    case "$subcommand" in
        rust|backend)
            log_section "运行 Rust 测试"
            cd "$PROJECT_ROOT"
            cargo test --workspace
            ;;
        frontend)
            log_section "运行前端测试"
            cd "$WEB_DIR"
            if [ -d "node_modules" ]; then
                npm test
            else
                log_warning "前端依赖未安装，跳过测试"
            fi
            ;;
        all)
            log_section "运行所有测试"
            cmd_test rust
            cmd_test frontend
            log_success "所有测试完成"
            ;;
        coverage)
            log_section "生成测试覆盖率报告"
            cmd_test_coverage
            ;;
        *)
            log_error "未知的 test 子命令: $subcommand"
            echo "可用的子命令: rust, frontend, all, coverage"
            exit 1
            ;;
    esac
}

cmd_test_coverage() {
    log_info "生成测试覆盖率报告..."

    # 检查是否安装了 cargo-tarpaulin
    if ! command -v cargo-tarpaulin &> /dev/null; then
        log_warning "cargo-tarpaulin 未安装，正在安装..."
        cargo install cargo-tarpaulin
    fi

    cd "$PROJECT_ROOT"
    cargo tarpaulin --workspace --out Html --out Stdout

    log_success "覆盖率报告生成完成"
    log_info "查看报告: target/tarpaulin-report.html"
}

# =============================================================================
# 清理命令
# =============================================================================
cmd_clean() {
    log_section "清理构建产物"

    cd "$PROJECT_ROOT"

    # 清理 Rust 构建
    log_info "清理 Rust 构建..."
    cargo clean

    # 清理前端构建
    if [ -d "$WEB_DIR/dist" ]; then
        log_info "清理前端构建..."
        rm -rf "$WEB_DIR/dist"
    fi

    if [ -d "$WEB_DIR/node_modules" ]; then
        log_info "清理前端依赖..."
        rm -rf "$WEB_DIR/node_modules"
    fi

    # 清理静态文件
    if [ -d "$STATIC_DIR" ]; then
        log_info "清理静态文件..."
        rm -rf "$STATIC_DIR"
    fi

    # 清理 dist 目录
    if [ -d "$DIST_DIR" ]; then
        log_info "清理 dist 目录..."
        rm -rf "$DIST_DIR"
    fi

    # 清理测试数据
    if [ -d "$PROJECT_ROOT/test_data" ]; then
        log_info "清理测试数据..."
        rm -rf "$PROJECT_ROOT/test_data"
    fi

    log_success "清理完成"
}

# =============================================================================
# 安装命令
# =============================================================================
cmd_install() {
    log_section "安装依赖"

    # 安装 Rust 依赖
    log_info "安装 Rust 依赖..."
    cd "$PROJECT_ROOT"
    cargo fetch

    # 安装前端依赖
    log_info "安装前端依赖..."
    cd "$WEB_DIR"
    npm install

    log_success "依赖安装完成"
}

# =============================================================================
# 代码检查命令
# =============================================================================
cmd_lint() {
    local subcommand=${1:-all}

    case "$subcommand" in
        rust|backend)
            log_section "运行 Rust Clippy"
            cd "$PROJECT_ROOT"
            cargo clippy --workspace -- -D warnings
            log_success "Rust 代码检查通过"
            ;;
        frontend)
            log_section "运行前端代码检查"
            cd "$WEB_DIR"
            if [ -f "package.json" ] && grep -q "lint" "package.json"; then
                npm run lint
            else
                log_warning "前端没有配置 lint 脚本"
            fi
            ;;
        all)
            log_section "运行所有代码检查"
            cmd_lint rust
            cmd_lint frontend
            log_success "所有代码检查通过"
            ;;
        *)
            log_error "未知的 lint 子命令: $subcommand"
            echo "可用的子命令: rust, frontend, all"
            exit 1
            ;;
    esac
}

# =============================================================================
# 格式化命令
# =============================================================================
cmd_fmt() {
    local subcommand=${1:-all}

    case "$subcommand" in
        rust|backend)
            log_section "格式化 Rust 代码"
            cd "$PROJECT_ROOT"
            cargo fmt --all
            log_success "Rust 代码格式化完成"
            ;;
        frontend)
            log_section "格式化前端代码"
            cd "$WEB_DIR"
            if [ -f "package.json" ] && grep -q "format" "package.json"; then
                npm run format
            else
                log_warning "前端没有配置 format 脚本"
            fi
            ;;
        all)
            log_section "格式化所有代码"
            cmd_fmt rust
            cmd_fmt frontend
            log_success "所有代码格式化完成"
            ;;
        *)
            log_error "未知的 fmt 子命令: $subcommand"
            echo "可用的子命令: rust, frontend, all"
            exit 1
            ;;
    esac
}

# =============================================================================
# 完整检查命令
# =============================================================================
cmd_check() {
    log_section "运行完整检查"

    log_info "步骤 1/4: 代码格式化检查"
    cd "$PROJECT_ROOT"
    if ! cargo fmt --all -- --check; then
        log_error "代码格式化检查失败，请运行 './manage.sh fmt' 修复"
        exit 1
    fi

    log_info "步骤 2/4: 代码检查"
    if ! cmd_lint rust; then
        log_error "代码检查失败"
        exit 1
    fi

    log_info "步骤 3/4: 运行测试"
    if ! cmd_test rust; then
        log_error "测试失败"
        exit 1
    fi

    log_info "步骤 4/4: 构建检查"
    if ! cargo build --workspace; then
        log_error "构建失败"
        exit 1
    fi

    log_success "完整检查通过！"
}

# =============================================================================
# 帮助命令
# =============================================================================
cmd_help() {
    cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════════════╗
║                    Agent Playground - 项目管理脚本                        ║
╚═══════════════════════════════════════════════════════════════════════════╝

用法: ./manage.sh <command> [subcommand] [options]

核心命令:
  dev                          开发模式（本地进程启动所有服务，热重载）

项目命令:
  build [dev|release|frontend|backend]  构建项目
  run                                    单独运行 API 服务
  test [rust|frontend|all|coverage]      运行测试
  clean                                  清理所有构建产物
  install                                安装所有依赖
  lint [rust|frontend|all]               代码检查
  fmt [rust|frontend|all]                代码格式化
  check                                  完整检查（lint + test + build）
  help                                   显示此帮助信息

详细说明:

  dev
    dev  - 本地进程启动所有服务（API + 前端 + 数据库），支持热重载
           前端: http://localhost:5173 (Vite 热重载)
           API:  http://localhost:8080
           API 文档: http://localhost:8080/api/docs

  build
    dev      - 开发模式构建（默认）
    release  - 生产模式构建
    frontend - 仅构建前端
    backend  - 仅构建后端

  test
    rust     - 运行 Rust 测试
    frontend - 运行前端测试
    all      - 运行所有测试（默认）
    coverage - 生成测试覆盖率报告

环境变量:
  API_PORT     - API 服务器端口 (默认: 自动检测可用端口)
  RUST_LOG     - 日志级别 (默认: info)
  CLEAN_BUILD  - 清理构建缓存后启动 (设置 CLEAN_BUILD=1 启用)

开发模式说明:
  • dev 模式使用 cargo watch 监视文件变化，自动重新编译 Rust 代码
  • 修改 crates/ 目录下的 .rs 文件会触发 API 服务自动重载
  • 修改 web/src/ 目录下的文件会触发前端热重载
  • 如果修改后没有生效，使用: CLEAN_BUILD=1 ./manage.sh dev

示例:
  ./manage.sh dev                    # 开发模式（本地热重载）
  ./manage.sh check                  # 运行完整检查

工作流示例:

  1. 首次开发:
     ./manage.sh install              # 安装依赖
     ./manage.sh dev                  # 启动所有服务
     # 按 Ctrl+C 停止 API 和前端

  2. 日常开发:
     ./manage.sh run                  # 只运行 API
     # 或同时运行前端: cd web && npm run dev

EOF
}

# =============================================================================
# 版本信息
# =============================================================================
show_version() {
    echo "Agent Playground Manager v0.1.0"
}

# =============================================================================
# 主函数
# =============================================================================
main() {
    # 加载环境变量
    load_env

    # 解析命令
    local command=${1:-help}
    shift || true

    case "$command" in
        dev)
            cmd_dev "$@"
            ;;
        build)
            cmd_build "$@"
            ;;
        run)
            cmd_run "$@"
            ;;
        test)
            cmd_test "$@"
            ;;
        clean)
            cmd_clean "$@"
            ;;
        install)
            cmd_install "$@"
            ;;
        lint)
            cmd_lint "$@"
            ;;
        fmt|format)
            cmd_fmt "$@"
            ;;
        check)
            cmd_check "$@"
            ;;
        version|-v|--version)
            show_version
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

# 运行主函数
main "$@"
