# Agent Playground 一站式编译部署方案

## 概述

本项目实现了完整的一站式编译和部署方案：

1. **一站式编译脚本**: `build.sh` (Unix/Linux/Mac) 和 `build.bat` (Windows)
2. **Rust 静态文件服务**: 使用 `rust-embed` 将前端嵌入二进制
3. **前后端统一部署**: 单个二进制文件包含所有内容

## 快速开始

```bash
# 开发模式构建 (推荐开发使用)
./build.sh dev

# 生产模式构建 (推荐部署使用)
./build.sh release

# 构建并运行
./build.sh run

# 开发监视模式 (自动重建)
./build.sh watch
```

## 项目结构

```
agent-playground/
├── build.sh                 # Unix/Linux/Mac 构建脚本
├── build.bat                # Windows 构建脚本
├── DEVELOPMENT.md           # 完整开发文档
├── web/                     # React + TypeScript 前端
│   ├── src/                 # 前端源码
│   └── vite.config.ts       # 构建输出到 crates/api/static/
├── crates/
│   └── api/                 # Rust Web 服务器
│       ├── src/main.rs      # 服务器入口
│       └── static/          # 前端构建输出目录
└── target/
    └── release/api          # 生产二进制文件
```

## 构建脚本命令

### Unix/Linux/Mac (build.sh)

```bash
./build.sh [命令]

命令:
  dev       # 开发模式构建 (快速，调试信息)
  release   # 生产模式构建 (优化，用于部署)
  frontend  # 仅构建前端
  backend   # 仅构建后端
  clean     # 清理所有构建产物
  run       # 构建并运行服务
  watch     # 开发模式 + 文件监视
  help      # 显示帮助
```

### Windows (build.bat)

```cmd
build.bat [命令]

命令与 Unix 版本相同
```

## 运行方式

### 方式 1: 使用构建脚本

```bash
./build.sh run
```

### 方式 2: 直接使用 Cargo

```bash
# 开发模式
cargo run -p api

# 生产模式
cargo run -p api --release
```

### 方式 3: 运行二进制文件

```bash
# 开发版本
./target/debug/api

# 生产版本
./target/release/api
```

## 配置选项

### 命令行参数

```bash
# 查看帮助
./target/release/api --help

# 自定义端口
./target/release/api --bind 0.0.0.0:3000

# API 模式 (不提供静态文件)
./target/release/api --api-only

# 自定义静态文件目录
./target/release/api --static-dir /path/to/static
```

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `API_BIND` | `0.0.0.0:8080` | 服务器绑定地址 |
| `RUST_LOG` | `info` | 日志级别 |
| `STATIC_DIR` | `crates/api/static` | 静态文件目录 |
| `API_ONLY` | - | 设置后仅提供 API |

## 访问服务

启动后访问以下地址：

- **Web UI**: http://localhost:8080
- **API 文档**: http://localhost:8080/api/docs
- **健康检查**: http://localhost:8080/api/health
- **版本信息**: http://localhost:8080/api/version

## 部署方案

### 方案 1: 独立二进制部署

```bash
# 构建生产版本
./build.sh release

# 复制二进制到服务器
scp target/release/api user@server:/opt/agent-playground/

# 在服务器上运行
ssh user@server "cd /opt/agent-playground && ./api"
```

### 方案 2: Docker 部署

FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN ./build.sh release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/api .
EXPOSE 8080
CMD ["./api"]
```

构建和运行：

```bash
```

### 方案 3: Systemd 服务

创建 `/etc/systemd/system/agent-playground.service`:

```ini
[Unit]
Description=Agent Playground
After=network.target

[Service]
Type=simple
User=agent
WorkingDirectory=/opt/agent-playground
ExecStart=/opt/agent-playground/target/release/api
Restart=always
Environment=RUST_LOG=info
Environment=API_BIND=0.0.0.0:8080

[Install]
WantedBy=multi-user.target
```

启用和启动服务：

```bash
sudo systemctl enable agent-playground
sudo systemctl start agent-playground
sudo systemctl status agent-playground
```

## 静态文件服务说明

### 开发模式

- 从 `crates/api/static/` 目录读取文件
- 支持热重载（文件修改后刷新即可看到）
- 适合开发调试

### 发布模式

- 使用 `rust-embed` 将静态文件嵌入二进制
- 单文件部署，无需额外静态资源
- 适合生产环境

### 前端路由支持

- 支持 React Router 等前端路由
- 访问不存在的路径自动返回 `index.html`
- API 路径 `/api/*` 不受影响

## API 端点

### 系统端点

- `GET /api/health` - 健康检查
- `GET /api/version` - 版本信息
- `GET /api/docs` - API 文档

### Brain API

- `GET /api/brain/knowledge` - 知识切片列表
- `GET /api/brain/slices` - 所有切片

### Engine API

- `GET /api/engine/sessions` - 会话列表
- `GET /api/engine/environments` - 环境列表

### Synergy API

- `GET /api/synergy/agents` - Agent 列表
- `GET /api/synergy/tasks` - 任务列表

## 故障排除

### 端口被占用

```bash
# 使用不同端口
./target/release/api --bind 0.0.0.0:3000
```

### 静态文件不更新

```bash
# 清理并重新构建
./build.sh clean
./build.sh dev
```

### 前端构建失败

```bash
# 清理前端依赖并重新安装
cd web
rm -rf node_modules package-lock.json
npm install
cd ..
./build.sh dev
```

## 开发工作流

### 推荐开发模式

终端 1 - 启动监视模式：
```bash
./build.sh watch
```

终端 2 - 访问应用：
```bash
open http://localhost:8080
```

### 前端独立开发

```bash
cd web
npm run dev
# 访问 http://localhost:5173
# API 请求自动代理到 localhost:8080
```

## 技术栈

- **后端**: Rust + Axum + Tokio
- **前端**: React + TypeScript + Vite + Tailwind CSS
- **静态文件**: rust-embed
- **构建**: 自定义 Shell/Batch 脚本

## 许可证

MIT License
