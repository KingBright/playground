# Agent Playground 开发指南

## 快速开始

### 环境要求

- **Rust**: 1.75+ (推荐安装 via [rustup](https://rustup.rs/))
- **Node.js**: 18+ (推荐安装 via [nvm](https://github.com/nvm-sh/nvm))
- **Git**: 2.x+

### 一键构建

```bash
# 克隆项目
git clone <repository-url>
cd agent-playground

# 一站式构建并运行
./build.sh dev      # 开发模式
./build.sh release  # 生产模式

# 或者使用 watch 模式（开发推荐）
./build.sh watch
```

### 手动构建步骤

如果不想使用脚本，可以手动构建：

```bash
# 1. 构建前端
cd web
npm install
npm run build
cd ..

# 2. 构建并运行后端
cargo build -p api
cargo run -p api
```

## 项目结构

```
agent-playground/
├── build.sh                  # Unix/Linux/Mac 构建脚本
├── build.bat                 # Windows 构建脚本
├── Cargo.toml               # Rust Workspace 配置
├── web/                     # 前端项目
│   ├── src/                 # TypeScript/React 源码
│   ├── package.json
│   └── vite.config.ts
├── crates/                  # Rust crates
│   ├── api/                 # Web 服务器入口
│   │   ├── src/
│   │   │   ├── main.rs      # 服务器入口
│   │   │   ├── server.rs    # 静态文件服务
│   │   │   └── routes/      # API 路由
│   │   └── static/          # 前端构建输出
│   ├── common/              # 共享类型和工具
│   ├── brain/               # External Brain
│   ├── engine/              # Agent Playground
│   └── synergy/             # Mission Control
└── DEVELOPMENT.md           # 本文档
```

## 构建脚本用法

### Unix/Linux/Mac

```bash
./build.sh [命令]

命令:
  dev       # 开发模式构建 (快速，包含调试信息)
  release   # 生产模式构建 (优化，用于部署)
  frontend  # 仅构建前端
  backend   # 仅构建后端
  clean     # 清理所有构建产物
  run       # 构建并运行服务
  watch     # 开发模式 + 文件监视自动重建
  help      # 显示帮助
```

### Windows

```cmd
build.bat [命令]

命令与 Unix 版本相同
```

## 开发工作流

### 推荐开发模式

```bash
# 终端 1: 启动 watch 模式（自动重建前后端）
./build.sh watch

# 终端 2: 访问应用
open http://localhost:8080
```

### API 开发

```bash
# 仅运行后端（API 模式）
cargo run -p api -- --api-only

# 查看帮助
cargo run -p api -- --help
```

### 前端开发

```bash
cd web
npm run dev  # 启动 Vite 开发服务器

# 前端开发服务器会代理 API 请求到 localhost:8080
# 查看 vite.config.ts 中的 proxy 配置
```

## 静态文件服务

### 工作原理

1. **开发模式**: 从 `crates/api/static/` 目录读取文件
2. **发布模式**: 使用 `rust-embed` 将静态文件嵌入二进制

### 文件位置

- **开发**: 前端构建输出到 `crates/api/static/`
- **发布**: 静态文件编译进 `target/release/api` 二进制

### 配置选项

```bash
# 自定义静态文件目录
cargo run -p api -- --static-dir /path/to/static

# API 模式（不提供静态文件）
cargo run -p api -- --api-only

# 自定义端口
cargo run -p api -- --bind 0.0.0.0:3000
```

## API 端点

### 系统端点

- `GET /api/health` - 健康检查
- `GET /api/version` - 版本信息
- `GET /api/docs` - API 文档

### Brain API

- `GET /api/brain/knowledge` - 知识切片列表
- `GET /api/brain/knowledge/search` - 知识搜索
- `GET /api/brain/slices` - 所有切片
- `GET /api/brain/collectors` - 数据采集器

### Engine API

- `GET /api/engine/sessions` - 会话列表
- `GET /api/engine/environments` - 环境列表
- `GET /api/engine/workflows` - 工作流列表

### Synergy API

- `GET /api/synergy/agents` - Agent 列表
- `GET /api/synergy/agents/registry` - 注册表
- `GET /api/synergy/tasks` - 任务列表
- `GET /api/synergy/missions` - 任务列表

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `API_BIND` | `0.0.0.0:8080` | 服务器绑定地址 |
| `RUST_LOG` | `info` | 日志级别 (trace/debug/info/warn/error) |
| `STATIC_DIR` | `crates/api/static` | 静态文件目录 |
| `API_ONLY` | - | 设置后仅提供 API，不服务静态文件 |

## 故障排除

### 前端构建失败

```bash
# 清理并重新安装依赖
cd web
rm -rf node_modules package-lock.json
npm install
npm run build
```

### 端口被占用

```bash
# 使用不同端口
cargo run -p api -- --bind 0.0.0.0:3000
```

### 静态文件不更新

```bash
# 清理并重新构建
./build.sh clean
./build.sh dev
```

## 部署

### 生产构建

```bash
# 完整优化构建
./build.sh release

# 二进制位置: target/release/api
```

### Docker 部署

```dockerfile
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

### Systemd 服务

```ini
# /etc/systemd/system/agent-playground.service
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

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

MIT License - 详见 LICENSE 文件
