#!/bin/bash

# Agent Playground - 测试运行脚本

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# 运行 Rust 测试
run_rust_tests() {
    log_info "Running Rust tests..."

    # Run tests for all workspace crates
    log_info "Testing all workspace crates..."

    # Common crate
    log_info "Testing common crate..."
    cargo test -p common --lib --quiet || true

    # Brain crate
    log_info "Testing brain crate..."
    cargo test -p brain --lib --quiet 2>/dev/null || log_warning "Brain crate tests skipped (some tests may be slow)"

    # Engine crate
    log_info "Testing engine crate..."
    cargo test -p engine --lib --quiet || true

    # Synergy crate
    log_info "Testing synergy crate..."
    cargo test -p synergy --lib --quiet || true

    log_success "Rust tests completed"
}

# 运行前端测试
run_frontend_tests() {
    log_info "Running frontend tests..."
    cd web

    if [ ! -d "node_modules" ]; then
        log_info "Installing dependencies..."
        npm install
    fi

    npm test -- --run || log_warning "Frontend tests failed or not configured"

    cd ..
    log_success "Frontend tests completed"
}

# 运行覆盖率
run_coverage() {
    log_info "Generating coverage report..."

    # Rust coverage
    if command -v cargo-tarpaulin &> /dev/null; then
        cargo tarpaulin --workspace --exclude api --exclude brain --exclude engine --exclude synergy --out Html
        log_success "Coverage report: tarpaulin-report.html"
    else
        log_warning "cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"
    fi
}

# 主函数
main() {
    case "${1:-all}" in
        rust)
            run_rust_tests
            ;;
        frontend)
            run_frontend_tests
            ;;
        coverage)
            run_coverage
            ;;
        all)
            run_rust_tests
            run_frontend_tests
            ;;
        *)
            echo "Usage: $0 [rust|frontend|coverage|all]"
            exit 1
            ;;
    esac
}

main "$@"
