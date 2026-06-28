+++
title = "核心概念"
description = """目标读者：希望从概念层面理解 shittim-chest 设计的开发者。"""
lang = "zhs"
category = "guides"
subcategory = "webui"
+++

# 核心概念

> **目标读者**：希望从概念层面理解 shittim-chest 设计的开发者。
> **最后更新**：2026-05-25

## 双仓库架构

shittim-chest 和 [entelecheia](https://github.com/celestia-island/entelecheia) 构成了一个双仓库系统，具有明确的边界：

- **entelecheia** — 智能体编排核心（scepter、13 个智能体、Cosmos/IEPL 运行时）。管理身份、权限和智能体配置。
- **shittim-chest** — 面向用户的外壳。管理认证、会话、聊天数据、LLM 提供商配置、前端 UI 以及到 scepter 的代理桥接。

它们通过 JWT 认证的 HTTP 和 WebSocket 通信。两者不直接访问对方的数据库。这种分离使得每个仓库可以独立开发、部署和扩展。

## 双运行模式

shittim-chest 支持两种运行模式：

### 独立模式

通过自己的 LLM 路由层独立运行。支持：

- 流式响应聊天（SSE + WebSocket）
- 通过配置的提供商进行图像生成
- 用户认证（密码 + GitHub OAuth）
- 提供商管理（添加/删除 LLM 提供商）

不需要 entelecheia。适用于开发和简单部署。

### 代理模式

作为进入 entelecheia 智能体系统的网关。增加：

- 带 JWT 传递的请求转发到 scepter
- 基于智能体聊天的 WebSocket 桥接
- Webhook 入口和触发器转发
- 通过 polemos 进行远程设备管理
- RBAC 权限查询和缓存

需要运行中的 entelecheia 实例。两种模式可以共存——独立 LLM 用于简单聊天，代理用于智能体编排。

## 认证模型

认证使用 `shittim_chest` 签发的 JWT 令牌：

1. **凭据存储**：密码（argon2 哈希）、会话、刷新令牌和 API 密钥存储在 `shittim_chest_db` 中。
1. **GitHub OAuth**：用户可以使用 GitHub 登录；首次登录时自动创建账户。
1. **权限存储**：用户分组、角色和权限矩阵存储在 `entelecheia_db` 中。
1. **JWT 流程**：登录时，`shittim_chest` 在本地验证凭据，然后从 scepter 获取权限。签发的 JWT 包含 `{ sub: user_id, groups: [...] }`。
1. **共享密钥**：JWT 签名密钥与 scepter 共享，因此两个服务可以独立验证令牌。
1. **令牌轮换**：访问令牌有效期为 1 小时；刷新令牌为 7 天。每次使用刷新令牌时进行轮换。

## 前端 (webui)

webui 是位于 `packages/webui/` 的统一前端，聊天界面在 `/`，管理面板在 `/backend`，使用 Vue 3 + Vite + Pinia（通过 `@vitejs/plugin-vue-jsx` 使用 TSX）构建。

## LLM 提供商系统

shittim-chest 拥有独立的 LLM 路由层：

- **提供商**：可配置的 LLM API 端点（兼容 OpenAI）。存储在 `shittim_chest_db` 中，API 密钥使用 AES-256-GCM 加密。
- **路由器**：多提供商路由，支持基于优先级的选路和自动故障转移。
- **类别**：提供商可标记为 `chat`、`image` 或两者。
- **管理**：通过 REST API 和 webui 管理面板进行完整 CRUD。可以测试提供商的连通性。
- **流式传输**：支持 SSE（简单、代理友好）和 WebSocket（双向）流式传输协议。

## 聊天系统

- **对话**：基于线程的聊天会话，包含标题和元数据
- **消息**：支持文本、图像和工具调用（函数调用）
- **流式传输**：通过 SSE 或 WebSocket 实时逐令牌传递响应
- **搜索**：使用 ILIKE 查询进行全文消息搜索
- **导出**：对话可以导出为 JSON 或 Markdown 格式
- **图像生成**：通过配置的提供商进行基于提示词的图像生成，支持"插入到聊天"功能

## 远程设备管理

shittim-chest 为 entelecheia/polemos 管理的远程设备提供基于浏览器的界面：

- **桌面**：基于 WebRTC 的远程桌面查看器，支持帧中继
- **终端**：基于 xterm.js 的终端模拟器，支持 WebSocket 中继
- **文件浏览器**：SFTP 文件浏览器后端（骨架）
- **信令**：基于 WebSocket 的 WebRTC 信令中继（SDP offer/answer、ICE 候选）

所有设备通信通过 entelecheia 的 polemos 智能体流转——shittim-chest 从不直接连接端点。

## 代理架构

`shittim_chest` 充当用户和 scepter 之间的网关：

- **HTTP 反向代理**：`/api/proxy/*` 将经过认证的请求转发到 scepter，并传递 JWT。
- **WebSocket 桥接**：聊天流式传输使用双向 WebSocket 转发（`浏览器 ↔ shittim_chest ↔ scepter`）。

这使得 `shittim_chest` 可以强制执行速率限制、记录使用情况并管理连接生命周期，而 scepter 无需处理单个浏览器连接。

## Webhook 管道

外部事件通过 Webhook 管道到达智能体核心：

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC 验证 → 解析事件 → 通过 Unix 套接字转发到 scepter → 智能体调度
```

每个提供商有自己的验证机制：

- **GitHub**：通过 `X-Hub-Signature-256` 进行 HMAC-SHA256
- **GitLab**：通过 `X-Gitlab-Token` 进行令牌验证
- **Gitee**：HMAC 配合令牌回退

附加功能：重复投递检测（LRU 缓存）、投递日志、IP 白名单和通用自定义 Webhook 端点。

## RBAC 模型

权限遵循基于分组的 RBAC 模型：

- **分组**：用户属于一个或多个分组。
- **角色**：分组有分配的角色。
- **权限**：每个角色定义了权限矩阵，涵盖：
  - 提供商配额（最大令牌数、最大请求数）
  - 智能体白名单（分组可以访问哪些智能体）
  - 管理能力（管理用户、配置提供商）

`shittim_chest` 在进程内缓存权限，带有 TTL（默认 5 分钟）。缓存在 TTL 过期、登出或从 scepter 传播的显式权限变更时失效。

## 前端策略

shittim-chest 采用两阶段前端方法：

**第一阶段（当前）**：Vue 3 前端（`webui`，位于 `packages/webui/`），使用 Vite + Pinia 构建，通过 `@vitejs/plugin-vue-jsx` 使用 TSX。它定义了 API 契约，并作为生产质量级的参考实现。

**第二阶段（未来）**：使用 Tairitsu 构建的 Rust → WASM 前端。旧版前端作为活文档和测试预言——相同的用户交互必须产生相同的结果。

## 类型安全桥接

TypeScript 类型通过外部的 `arona` 协议 crate 从 Rust 代码生成，确保前端与后端的一致性：

```text
arona Rust crate（git 依赖）
  → #[derive(ts_rs::TS)]
  → ts-rs codegen → packages/webui/src/types/arona/（TypeScript）
  → 由 webui 以 @celestia-island/arona 消费
```

这消除了手动类型同步。当 `arona` crate 中的 Rust 类型发生变化时，TypeScript 绑定会重新生成并被 webui 使用。
