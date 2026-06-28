+++
title = "架构深入解析"
description = """目标读者：需要了解 shittim-chest 内部工作方式的开发者。"""
lang = "zhs"
category = "guides"
subcategory = "webui"
+++

# 架构深入解析

> **目标读者**：需要了解 shittim-chest 内部工作方式的开发者。
> **最后更新**：2026-05-25

## 项目概览

shittim-chest 是 [entelecheia](https://github.com/celestia-island/entelecheia) 的**面向用户的外壳**，后者是一个基于 Rust 的多智能体协作平台。两者的边界经过精心设计：

- **entelecheia** 负责智能体编排（scepter、13 个智能体、Cosmos/IEPL 运行时）、身份认证和权限管理。
- **shittim-chest** 负责用户认证、会话管理、聊天数据、LLM 提供商配置、前端展示以及到 scepter 的代理桥接。

两者通过 JWT 认证的 HTTP 和 WebSocket 进行通信。shittim-chest 绝不直接访问 entelecheia 的数据库来进行智能体操作。

## 后端技术栈

### Axum 路由

核心后端（`packages/core`）是一个 Axum 0.8 应用程序。路由挂载了以下模块组：

```text
/                   → 健康检查
/api/auth/*         → AuthService（登录、注册、GitHub OAuth、刷新、登出）
/api/chat/*         → ChatService（对话、消息、SSE/WS 流式传输、搜索、导出）
/api/providers/*    → ProviderService（LLM 提供商 CRUD、API 密钥加密、测试）
/api/generation/*   → GenerationService（图像生成）
/api/devices/*      → DeviceService（远程设备列表、会话、信令）
/api/webhook/*      → WebhookService（GitHub、GitLab、Gitee、自定义；HMAC 验证）
/api/proxy/*        → ProxyService（HTTP 反向代理 + 到 scepter 的 WebSocket 桥接）
/static/*           → SPA 静态文件托管（仅生产环境）
```

### SeaORM + PostgreSQL

数据库访问使用 SeaORM 1.x 配合 PostgreSQL。`shittim_chest_db` 存储：

- 用户认证：密码哈希（argon2）、会话、刷新令牌、API 密钥、OAuth 连接
- 聊天数据：对话、消息
- LLM 提供商配置（API 密钥使用 AES-256-GCM 静态加密）
- 远程设备记录和设备会话
- 多平台消息的频道配置
- Webhook 投递日志

5 个迁移文件和 25 个实体模型位于 `packages/core/src/{migration,entity}/`。

### JWT 认证

`shittim_chest` 签发包含 `{ sub: user_id, groups: [...] }` 的 JWT。JWT 密钥与 scepter 共享，因此两个服务可以独立验证令牌。访问令牌有效期为 1 小时；刷新令牌有效期为 7 天，每次使用时会轮换。

## 独立的 LLM 能力

shittim-chest 拥有自己的 LLM 路由层，独立于 entelecheia 运行：

- **LlmRouter**：多提供商路由器，支持基于优先级的选路和故障转移
- **提供商管理**：用于添加/编辑/删除 LLM 提供商的 CRUD 端点
- **API 密钥加密**：提供商 API 密钥使用 AES-256-GCM 静态加密
- **兼容 OpenAI**：兼容任何 OpenAI 兼容的 API（DeepSeek、OpenAI、本地模型等）
- **双流式传输**：支持 SSE（服务器发送事件）和 WebSocket 流式传输的聊天响应

这意味着 shittim-chest 可以作为独立的聊天应用程序运行而无需 entelecheia，也可以通过代理层使用 entelecheia 智能体。

## 认证流程

### 登录流程

```text
用户 → shittim_chest: POST /api/auth/login { username, password }
shittim_chest → shittim_chest_db: SELECT user WHERE username = ?（验证 argon2 哈希）
shittim_chest → scepter: GET /api/user/{id}/permissions
scepter → entelecheia_db: 查询分组 + 权限
scepter → shittim_chest: { groups: [...], permissions: {...} }
shittim_chest → 用户: { access_token, refresh_token }
shittim_chest: 存储会话 + 缓存 RBAC
```

### GitHub OAuth

```text
用户 → shittim_chest: GET /api/auth/github
shittim_chest → 用户: 302 重定向到 GitHub OAuth
用户 → GitHub: 授权
GitHub → shittim_chest: GET /api/auth/github/callback?code=...
shittim_chest → GitHub: 用 code 交换 access token
shittim_chest → GitHub: GET /user（获取用户信息）
shittim_chest → shittim_chest_db: INSERT/UPDATE oauth_connections
shittim_chest → 用户: { access_token, refresh_token }（新用户自动创建）
```

## 聊天架构

### 消息流程（独立 LLM）

```text
用户 → POST /api/chat/conversations/:id/messages
shittim_chest: 验证 JWT，加载对话
shittim_chest → LlmRouter: 将请求路由到最佳提供商
LlmRouter → LLM 提供商: POST chat/completions（流式传输）
LLM 提供商 → LlmRouter: SSE 流
LlmRouter → 用户: SSE/WS 流（令牌实时到达）
shittim_chest: 将消息持久化到 shittim_chest_db
```

### SSE 与 WebSocket 流式传输

- **SSE**（`/api/chat/stream`）：简单的 HTTP 流式传输，兼容代理，自动重连
- **WebSocket**（`/ws/chat/stream`）：双向传输，支持取消和实时交互

## 代理架构

`/api/proxy/*` 端点将经过认证的请求转发到 scepter：

1. 浏览器携带 JWT 打开 `ws://shittim-chest:80/api/proxy/chat`
1. `shittim_chest` 验证 JWT，打开到 scepter 的连接并转发 JWT
1. 浏览器和 scepter 之间双向消息转发
1. `shittim_chest` 强制执行速率限制、记录使用情况、管理连接生命周期

## Webhook 管道

来自外部服务的 Webhook 通过 `/api/webhook/*` 入口：

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC 验证 → 解析事件 → 通过 Unix 套接字转发到 scepter
```

支持的来源：GitHub（HMAC-SHA256）、GitLab（令牌）、Gitee（HMAC + 令牌回退），以及通用的 `/api/webhook/custom/{name}` 端点。功能包括：

- 重复投递检测（LRU 缓存，10,000 个 ID）
- 投递日志及列表 API
- Webhook 来源的 IP 白名单

## 远程设备管理

远程设备通过信令中继进行管理：

```text
浏览器 (webui) → WS /api/devices/stream → shittim_chest（信令中继） → Unix 套接字 → entelecheia/polemos
```

功能：

- 通过 REST API 进行设备列表和会话 CRUD
- WebRTC 信令（SDP offer/answer、ICE 候选）
- 终端中继（WebSocket 到 xterm.js）
- 桌面帧中继
- SFTP 文件浏览器后端

shittim-chest 绝不直接连接远程设备——所有数据通过 entelecheia 的 polemos 智能体流转。

## 数据所有权

### shittim_chest_db

| 数据 | 表 | 原因 |
| --- | --- | --- |
| 密码哈希 (argon2) | `auth_users` | 展示层控制登录流程 |
| 活跃会话、刷新令牌 | `sessions` | 会话管理属于前端关注点 |
| 加密的 API 密钥 | `api_keys` | API 密钥签发面向用户 |
| OAuth 连接 | `oauth_connections` | 第三方认证绑定面向用户 |
| 对话、消息 | `conversations`、`messages` | 聊天数据面向用户 |
| LLM 提供商配置 | `llm_providers` | 提供商管理面向用户（密钥已加密） |
| 远程设备记录 | `remote_devices`、`device_sessions` | 设备追踪面向用户 |
| 频道配置 | `channel_configs` 等 | 多平台配置面向用户 |

### entelecheia_db

| 数据 | 原因 |
| --- | --- |
| 用户身份、分组、角色分配 | 核心层强制执行权限 |
| GroupPermissions（提供商配额、智能体白名单） | 智能体级别的策略随智能体存放 |
| 智能体配置、Cosmos/IEPL 状态 | 编排数据属于核心层 |

## 双前端策略

### 第一阶段：Vue 3（当前）

| 包 | 技术 | 端口 | 用途 |
| --- | --- | --- | --- |
| `webui` | Vue 3 + Vite + Pinia (TSX) | `:3000（共享）` | 统一 Web 界面：聊天、图像生成、设备、管理（提供商、智能体、RBAC、Webhook） |

### 第二阶段：Rust WASM（未来）

| 包 | 技术 | 用途 |
| --- | --- | --- |
| `webui` | Rust → WASM (Tairitsu) | 长期统一 Web 界面（聊天 + 管理） |

旧版前端作为活文档使用。在过渡期间，两个版本并行运行，相同的用户交互必须产生相同的结果。

## 反向代理部署模式

shittim-chest 支持三种反向代理模式，由 `.env` 中的 `SHITTIM_CHEST_PROXY_MODE` 控制。

### 模式 1：None（直连）

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=none   # 或不设置
```

核心服务器直接绑定到 `SHITTIM_CHEST_HOST:SHITTIM_CHEST_PORT`（默认 `0.0.0.0:80`）。无 TLS、无反向代理容器。适用于：

- 本地开发
- 已有反向代理（Cloudflare Tunnel、AWS ALB、Traefik 标签）之后的场景
- 由其他服务处理 TLS 终止的 Docker 网络

### 模式 2：Caddy 自动

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_DOMAIN=app.example.com
```

CLI 创建一个 `shittim-chest-caddy` 容器（镜像 `caddy:2`），该容器：

1. 监听端口 80/443（可通过 `SHITTIM_CHEST_PROXY_HTTP_PORT` / `SHITTIM_CHEST_PROXY_HTTPS_PORT` 配置）
1. 通过 Let's Encrypt（Caddy 内置的 ACME）自动配置 TLS 证书
1. 将所有请求代理到 Docker 网络上的核心后端

无需 Caddyfile——CLI 会自动生成。域名必须有指向主机的公共 DNS。

### 模式 3：Caddy 自定义

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/caddy/Caddyfile
SHITTIM_CHEST_PROXY_EXTRA_VOLUMES=/etc/letsencrypt:/etc/letsencrypt
```

相同的 Caddy 容器，但您提供自己的 Caddyfile（从主机挂载）。当需要以下功能时使用此模式：

- 多个虚拟主机
- 自定义 TLS 证书路径
- 额外的中间件（基本认证、速率限制等）
- 与 API 一起提供静态文件

### 模式 4：Nginx 自定义

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=nginx
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/nginx/conf.d/default.conf
```

创建一个带有您配置文件的 `nginx:bookworm` 容器。您需要自行管理 TLS 证书。适用于 Nginx 是标准的环境。

### 容器生命周期

所有代理容器由 CLI 通过 Docker API（`bollard`）管理：

| 命令 | 行为 |
| --- | --- |
| `just dev` / `chest up` | 如果设置了 `PROXY_MODE`，创建/启动代理容器 |
| `just dev-stop` / `chest down` | 停止并移除代理容器 |
| 容器已在运行 | 重用现有容器（幂等） |

代理容器加入与核心后端相同的 Docker 网络，因此可以通过内部主机名（`core` 或 `shittim-chest`）访问后端。
