# 构建阶段
FROM rust:1.75-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制 Cargo 配置
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/*/

# 创建虚拟 main.rs 来缓存依赖
RUN mkdir -p crates/api/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    mkdir -p crates/common/src && \
    echo "" > crates/common/src/lib.rs && \
    mkdir -p crates/brain/src && \
    echo "" > crates/brain/src/lib.rs && \
    mkdir -p crates/engine/src && \
    echo "" > crates/engine/src/lib.rs && \
    mkdir -p crates/synergy/src && \
    echo "" > crates/synergy/src/lib.rs

# 构建依赖（缓存层）
RUN cargo build --release -p api 2>/dev/null || true

# 复制实际源代码
COPY crates ./crates

# 构建应用
RUN touch crates/*/src/lib.rs crates/*/src/main.rs 2>/dev/null || true
RUN cargo build --release -p api

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 创建必要的目录
RUN mkdir -p /app/static /app/archive /app/data

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/api /app/api

# 复制静态文件（如果存在）
COPY --from=builder /app/crates/api/static /app/static

# 设置环境变量
ENV RUST_LOG=info
ENV API_PORT=8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# 暴露端口
EXPOSE 8080

# 启动命令
CMD ["./api"]
