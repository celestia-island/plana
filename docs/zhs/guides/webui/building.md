+++
title = "构建与开发指南"
description = """目标读者：搭建本地 shittim-chest 开发环境的贡献者。"""
lang = "zhs"
category = "guides"
subcategory = "webui"
+++

# 构建与开发指南

> **目标读者**：搭建本地 shittim-chest 开发环境的贡献者。
> **最后更新**：2026-05-25

## 前置条件

| 工具 | 最低版本 | 说明 |
| --- | --- | --- |
| Rust | 1.85+ | 需要 Edition 2024；通过 <https://rustup.rs> 安装 |
| Node.js | 20+ | 推荐 LTS 版本 |
| pnpm | 9+ | `corepack enable && corepack prepare pnpm@latest --activate` |
| just | 最新版 | 命令运行器；`cargo install just` |
| PostgreSQL | 18+ | 用于认证 + 聊天数据的 shittim_chest_db |
| entelecheia scepter | 可选 | 代理/设备功能需要；独立聊天模式可选 |

验证所有工具：

```bash
rustc --version    # >= 1.85
node --version     # >= 20
pnpm --version     # >= 9
just --version
psql --version     # >= 18
```

## 克隆与初始化

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
```

## 环境变量

克隆后编辑 `.env`。每个变量都有内联文档；以下是摘要。

### 服务器

| 变量 | 默认值 | 用途 |
| --- | --- | --- |
| `SHITTIM_CHEST_HOST` | `0.0.0.0` | 监听地址 |
| `SHITTIM_CHEST_PORT` | `80` | 监听端口 |

### 数据库

| 变量 | 默认值 | 用途 |
| --- | --- | --- |
| `SHITTIM_CHEST_DATABASE_URL` | `postgresql://sc:pass@localhost:5432/shittim_chest` | PostgreSQL 连接字符串 |
| `SHITTIM_CHEST_DATABASE_MAX_CONNECTIONS` | `10` | SeaORM 连接池大小 |

创建数据库和用户：

```sql
CREATE USER sc WITH PASSWORD 'pass';
CREATE DATABASE shittim_chest OWNER sc;
```

### JWT 与加密

| 变量 | 默认值 | 用途 |
| --- | --- | --- |
| `JWT_SECRET` | `change-me-in-production` | 与 scepter 共享的密钥；**必须一致** |
| `JWT_EXPIRATION_SECONDS` | `3600` | 访问令牌有效期（1 小时） |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | 刷新令牌有效期（7 天） |
| `SHITTIM_CHEST_ENCRYPTION_KEY` | `change-me-32-bytes-base64-encoded` | 用于 API 密钥和 OAuth 令牌的 AES-256-GCM 密钥 |

生成生产环境密钥：

```bash
openssl rand -base64 32
```

### LLM 提供商（独立运行）

设置这些变量可让 shittim-chest 在无 entelecheia 的情况下独立运行：

| 变量 | 用途 |
| --- | --- |
| `LLM_DEFAULT_PROVIDER_ENDPOINT` | OpenAI 兼容的 API 端点（例如 `https://api.deepseek.com/v1`） |
| `LLM_DEFAULT_PROVIDER_API_KEY` | 提供商的 API 密钥 |
| `LLM_DEFAULT_PROVIDER_MODELS` | 逗号分隔的模型列表（例如 `deepseek-chat,deepseek-reasoner`） |
| `LLM_DEFAULT_PROVIDER_CATEGORY` | 提供商类别：`chat` 或 `image` |
| `LLM_STREAM_BUFFER_SECONDS` | 流缓冲区超时（默认：60） |
| `LLM_MAX_TOKENS_DEFAULT` | 默认最大令牌数（默认：4096） |
| `LLM_REQUEST_TIMEOUT_SECONDS` | HTTP 请求超时（默认：120） |

### 远程设备

| 变量 | 默认值 | 用途 |
| --- | --- | --- |
| `REMOTE_DEVICES_ENABLED` | `false` | 启用远程设备功能 |
| `REMOTE_DEVICES_SCEPTER_SOCK` | `/run/entelecheia/device_stream.sock` | 设备数据的 Unix 套接字 |
| `REMOTE_DEVICES_FRAME_BUFFER_SIZE` | `4194304` | 帧缓冲区大小（字节） |
| `REMOTE_DEVICES_MAX_SESSIONS_PER_USER` | `3` | 最大并发设备会话数 |
| `WEBRTC_ICE_SERVERS` | `stun:stun.l.google.com:19302` | ICE 服务器列表 |

### GitHub OAuth

| 变量 | 用途 |
| --- | --- |
| `GITHUB_CLIENT_ID` | GitHub OAuth App 客户端 ID |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth App 客户端密钥 |
| `GITHUB_REDIRECT_URI` | OAuth 回调 URL（例如 `https://your-domain/api/auth/github/callback`） |

### Scepter 连接（用于代理功能）

| 变量 | 默认值 | 用途 |
| --- | --- | --- |
| `ENTELECHEIA_SCEPTER_URL` | `http://localhost:8424` | scepter 的 HTTP 端点 |
| `ENTELECHEIA_SCEPTER_WS_URL` | `ws://localhost:8424` | scepter 的 WebSocket 端点 |
| `ENTELECHEIA_TUI_SOCK` | `/run/entelecheia/entelecheia.sock` | 用于触发器转发的 Unix 套接字 |

### Webhook

| 变量 | 用途 |
| --- | --- |
| `WEBHOOK_GITHUB_SECRET` | GitHub Webhook 验证的 HMAC 密钥 |
| `WEBHOOK_GITLAB_SECRET` | GitLab Webhook 验证的令牌 |
| `WEBHOOK_PUBLIC_URL` | Webhook 端点的公开访问 URL |

## 数据库设置

```bash
just db-init      # 创建 schema（运行 SeaORM 迁移）
just db-migrate   # 应用待处理的迁移
```

### Schema 概览

`shittim_chest_db` 管理面向用户的数据：

| 表 | 用途 |
| --- | --- |
| `auth_users` | 使用 argon2 密码哈希的用户账户 |
| `sessions` | 包含刷新令牌的活跃会话 |
| `api_keys` | API 密钥记录（已哈希） |
| `oauth_connections` | 第三方 OAuth 绑定（GitHub） |
| `conversations` | 聊天会话 |
| `messages` | 包含工具调用数据的聊天消息 |
| `llm_providers` | LLM 提供商配置（API 密钥已加密） |
| `remote_devices` | 远程设备记录 |
| `device_sessions` | 活跃设备会话 |
| `channel_configs` | 多平台频道配置 |
| `channel_messages` | 频道消息记录 |
| `channel_pairings` | 频道到聊天的配对 |

重置数据库：

```bash
just db-reset
```

## 后端开发

```bash
just dev-backend
```

这会运行 `cargo run --package shittim_chest`。服务器在 `:80` 启动。

### CLI 命令

```bash
shittim_chest db-init      # 创建数据库 schema
shittim_chest db-migrate   # 应用待处理的迁移
shittim_chest db-reset     # 删除并重建 schema
shittim_chest server       # 启动 Web 服务器
```

### 热重载

```bash
cargo install cargo-watch
cargo watch -x 'run --package shittim_chest -- server'
```

### API 端点概览

| 路由组 | 用途 |
| --- | --- |
| `/api/auth/*` | 登录、注册、GitHub OAuth、刷新、登出 |
| `/api/chat/*` | 对话、消息、SSE/WS 流式传输、搜索、导出 |
| `/api/providers/*` | LLM 提供商 CRUD、API 密钥管理、测试 |
| `/api/generation/*` | 图像生成、模型列表 |
| `/api/devices/*` | 远程设备列表、会话、WebRTC 信令 |
| `/api/webhook/*` | GitHub/GitLab/Gitee/自定义 Webhook 入口 |
| `/api/proxy/*` | 到 scepter 的反向代理（HTTP + WebSocket） |
| `/static/*` | SPA 静态文件托管 |

## 前端开发

### 安装依赖

```bash
pnpm install
```

### webui

```bash
just dev    # 构建前端 + 在 :3000 启动后端
just watch  # 文件变更时自动重建
```

两个前端均由 Vite 构建到 `dist/`。后端在 `:3000` 直接提供这些静态文件——无需单独的 Vite 开发服务器或代理。在开发模式下，`dev.py` 会监视前端源码并自动重建。

## 跨项目设置

要使用共享的 `arona` 协议 crate 进行本地开发，请将其补丁到本地检出目录。编辑 `~/.cargo/config.toml`（切勿提交到仓库）：

```toml
[patch.'https://github.com/celestia-island/arona']
arona = { path = "/path/to/arona" }
```

对于 npm，webui 通过 `@celestia-island/arona` 路径别名消费 `arona` crate 的 TypeScript 绑定，指向 `packages/webui/src/types/arona/`。

## 生产环境构建

```bash
just build
```

这会运行 `cargo build --release` 和 `pnpm run build:all`。输出位置：

- 后端二进制文件：`target/release/shittim_chest`
- 前端资源：`packages/webui/dist/`

### Docker

使用 CLI 封装构建和运行（直接使用 Docker API）：

```bash
just dev
```

或手动操作：

```bash
just build        # 构建 Docker 镜像
just up           # 启动所有服务
just migrate      # 运行数据库迁移
```

生产环境二进制文件通过 Axum 的静态文件中间件在 `/` 提供前端资源。无需单独的前端服务器。

## 常见问题

### 数据库连接被拒绝

```text
error: connection to server at "localhost", port 5432 failed
```

**解决方法**：确保 PostgreSQL 正在运行，且 `.env` 中的 `SHITTIM_CHEST_DATABASE_URL` 与您的设置匹配。使用 `psql "$SHITTIM_CHEST_DATABASE_URL" -c 'SELECT 1'` 验证。

### Scepter 不可达

```text
error: error sending request for url (http://localhost:8424/...)
```

**解决方法**：启动 entelecheia scepter 实例，或配置 LLM 提供商以独立模式运行。后端在无 scepter 的情况下仍可进行聊天/图像生成。

### 浏览器中出现 CORS 错误

```text
Access-Control-Allow-Origin header is present on the requested resource
```

**解决方法**：开发后端对 `localhost` 来源启用了 CORS。如果更改了端口，请更新 CORS 配置。生产部署应配置反向代理（nginx/caddy）来处理 CORS。

### pnpm install 失败

**解决方法**：确保使用 pnpm 9+。运行 `corepack enable && corepack prepare pnpm@latest --activate` 设置正确的版本。

### cargo build 在共享 crate 上失败

**解决方法**：如果在 `~/.cargo/config.toml` 中有本地补丁，请确保路径存在且 crate 名称匹配。移除补丁部分以改用 git 依赖。
