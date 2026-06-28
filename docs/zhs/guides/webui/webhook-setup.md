+++
title = "Webhook 配置指南"
description = """目标读者：将外部服务与 shittim-chest 集成的管理员。"""
lang = "zhs"
category = "guides"
subcategory = "webui"
+++

# Webhook 配置指南

> **目标读者**：将外部服务与 shittim-chest 集成的管理员。
> **最后更新**：2026-05-25

## 概述

Webhook 允许外部服务（GitHub、GitLab、Gitee）将实时事件发送到 shittim-chest。事件经过验证、解析后转发到 scepter，由 scepter 分发给相应的智能体。

```text
外部服务 → shittim_chest → scepter → 智能体
```

`shittim_chest` 还支持不原生支持的自定义 Webhook 端点。

## GitHub Webhook 设置

### 步骤 1：配置环境

在 `.env` 中设置 Webhook 密钥：

```bash
WEBHOOK_GITHUB_SECRET=your-hmac-secret-here
WEBHOOK_PUBLIC_URL=https://your-domain.com
```

生成一个强密钥：

```bash
openssl rand -hex 32
```

### 步骤 2：在 GitHub 中创建 Webhook

1. 导航到您的仓库 → **Settings** → **Webhooks** → **Add webhook**
1. 将 **Payload URL** 设置为 `https://your-domain.com/api/webhook/github`
1. 将 **Content type** 设置为 `application/json`
1. 将 **Secret** 设置为与 `WEBHOOK_GITHUB_SECRET` 相同的值
1. 选择事件：`push`、`pull_request`、`issues`、`issue_comment`
1. 确保 **Active** 已勾选
1. 点击 **Add webhook**

### 步骤 3：验证

GitHub 会立即发送一个 `ping` 事件。检查 **Recent Deliveries** 选项卡确认返回了 `200` 响应。

## GitLab Webhook 设置

### 步骤 1：配置环境

```bash
WEBHOOK_GITLAB_SECRET=your-gitlab-secret-token
```

### 步骤 2：在 GitLab 中创建 Webhook

1. 导航到您的项目 → **Settings** → **Webhooks**
1. 将 **URL** 设置为 `https://your-domain.com/api/webhook/gitlab`
1. 将 **Secret token** 设置为与 `WEBHOOK_GITLAB_SECRET` 相同的值
1. 选择触发器：`Push events`、`Merge request events`、`Issue events`
1. 确保 **Enable SSL verification** 已勾选（HTTPS 环境下）
1. 点击 **Add webhook**

### 步骤 3：验证

使用 GitLab 中的 **Test** 按钮发送测试事件。确认投递成功。

## Gitee Webhook 设置

Gitee（码云）Webhook 同样受支持。

### 步骤 1：配置环境

Gitee 使用相同的 `WEBHOOK_GITLAB_SECRET` 进行 HMAC 验证（令牌作为回退）。或者，如果使用基于密码的认证，设置 `WEBHOOK_GITEE_PASSWORD`。

### 步骤 2：在 Gitee 中创建 Webhook

1. 导航到您的仓库 → **管理** → **Webhooks**
1. 将 **URL** 设置为 `https://your-domain.com/api/webhook/gitee`
1. 将 **密码/签名密钥** 设置为相同的密钥
1. 选择事件：`Push`、`Pull Request`、`Issues`
1. 点击 **添加**

## 自定义 Webhook

`shittim_chest` 在 `/api/webhook/custom/{name}` 支持通用自定义 Webhook 端点。要添加自定义 Webhook 来源：

1. 在 `.env` 中设置 `WEBHOOK_PUBLIC_URL`
1. 配置外部服务将 POST 请求发送到 `https://your-domain.com/api/webhook/custom/{name}`
1. 事件将以该 Webhook 名称作为事件来源转发到 scepter

在代码层面集成新的 Webhook 提供商：

1. 在 `packages/core/src/webhook.rs` 中添加处理器
1. 为新提供商实现 HMAC 或令牌验证
1. 解析自定义事件格式并通过 Unix 套接字转发到 scepter

## IP 白名单

`shittim_chest` 支持 Webhook 来源的 IP 白名单，以拒绝来自未知来源的请求：

```bash
# .env
WEBHOOK_IP_WHITELIST=140.82.112.0/20,192.30.252.0/22  # GitHub IP
```

为每个 Webhook 提供商配置 CIDR 范围。来自白名单外 IP 的请求将被拒绝。

## 事件类型

支持的事件及其到 scepter 触发器的映射：

| 来源 | 事件 | scepter `event_type` |
| --- | --- | --- |
| GitHub | `push` | `github.push` |
| GitHub | `pull_request` | `github.pull_request` |
| GitHub | `issues` | `github.issues` |
| GitHub | `issue_comment` | `github.issue_comment` |
| GitLab | `push` | `gitlab.push` |
| GitLab | `merge_request` | `gitlab.merge_request` |
| GitLab | `issues` | `gitlab.issues` |
| Gitee | `push` | `gitee.push` |
| Gitee | `pull_request` | `gitee.pull_request` |
| Gitee | `issues` | `gitee.issues` |

## 投递日志

`shittim_chest` 维护 Webhook 事件的投递日志。使用 LRU 缓存（最多 10,000 个投递 ID）检测重复投递。可通过以下方式访问投递日志：

- **REST API**：`GET /api/webhook/deliveries`
- 管理面板：**Webhooks** → **Delivery Log**

## 安全

所有 Webhook 必须通过签名验证：

- **GitHub**：使用 `X-Hub-Signature-256` 头部。根据 `WEBHOOK_GITHUB_SECRET` 验证。
- **GitLab**：使用 `X-Gitlab-Token` 头部。根据 `WEBHOOK_GITLAB_SECRET` 验证。
- **Gitee**：使用 HMAC-SHA256 签名，配合令牌回退。

没有有效签名的请求将被拒绝，返回 `401 Unauthorized`。切勿在客户端代码或日志中暴露 Webhook 密钥。

## 测试

使用管理面板测试 Webhook 集成：

1. 登录管理面板（默认 `:3000`）
1. 在侧边栏导航到 **Webhooks**
1. 查看投递日志和配置
1. 通过外部服务的测试功能测试端点

您也可以使用 curl 手动测试：

```bash
curl -X POST https://your-domain.com/api/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=<computed-hmac>" \
  -d '{"action":"push","ref":"refs/heads/main"}'
```

## 故障排除

### 401 Unauthorized

**原因**：HMAC 签名不匹配或 IP 不在白名单中。
**解决方法**：确保 `.env` 中的密钥与来源平台配置的密钥匹配。检查尾部空白或编码问题。验证 IP 白名单配置。

### 502 Bad Gateway

**原因**：scepter 不可达。
**解决方法**：验证 `.env` 中的 `ENTELECHEIA_SCEPTER_URL` 和 `ENTELECHEIA_TUI_SOCK`。确保 scepter 实例正在运行，且 Unix 套接字路径可访问。

### 事件未到达智能体

**原因**：事件类型未映射或智能体未配置处理该事件。
**解决方法**：检查后端日志中解析的 `event_type`。验证目标智能体已为该事件注册了处理器。通过 API 或管理面板检查投递日志。

### 重复投递

**原因**：外部服务由于超时正在重试。`shittim_chest` 自动通过 LRU 缓存检测重复。
**解决方法**：如果合法的重试被阻止，增加投递 ID 缓存大小。确保 `shittim_chest` 在服务的超时窗口内响应（GitHub：10 秒）。
