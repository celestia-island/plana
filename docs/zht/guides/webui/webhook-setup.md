+++
title = "Webhook 設定指南"
description = """> 目標讀者：將外部服務與 shittim-chest 整合的管理員。"""
lang = "zht"
category = "guides"
subcategory = "webui"
+++

# Webhook 設定指南

> **目標讀者**：將外部服務與 shittim-chest 整合的管理員。
> **最後更新**：2026-05-25

## 總覽

Webhook 允許外部服務（GitHub、GitLab、Gitee）向 shittim-chest 發送即時事件。事件經過驗證、解析後轉發到 scepter，scepter 會將其分派給適當的代理。

```text
外部服務 → shittim_chest → scepter → 代理
```

`shittim_chest` 也支援非原生支援服務的自訂 webhook 端點。

## GitHub Webhook 設定

### 步驟 1：設定環境

在您的 `.env` 中設定 webhook 金鑰：

```bash
WEBHOOK_GITHUB_SECRET=your-hmac-secret-here
WEBHOOK_PUBLIC_URL=https://your-domain.com
```

生成一個強金鑰：

```bash
openssl rand -hex 32
```

### 步驟 2：在 GitHub 中建立 Webhook

1. 前往您的倉庫 → **Settings** → **Webhooks** → **Add webhook**
1. 設定 **Payload URL** 為 `https://your-domain.com/api/webhook/github`
1. 設定 **Content type** 為 `application/json`
1. 設定 **Secret** 與 `WEBHOOK_GITHUB_SECRET` 的值相同
1. 選取事件：`push`、`pull_request`、`issues`、`issue_comment`
1. 確保 **Active** 已勾選
1. 點擊 **Add webhook**

### 步驟 3：驗證

GitHub 會立即發送一個 `ping` 事件。檢查 **Recent Deliveries** 頁籤以確認 `200` 回應。

## GitLab Webhook 設定

### 步驟 1：設定環境

```bash
WEBHOOK_GITLAB_SECRET=your-gitlab-secret-token
```

### 步驟 2：在 GitLab 中建立 Webhook

1. 前往您的專案 → **Settings** → **Webhooks**
1. 設定 **URL** 為 `https://your-domain.com/api/webhook/gitlab`
1. 設定 **Secret token** 與 `WEBHOOK_GITLAB_SECRET` 的值相同
1. 選取觸發器：`Push events`、`Merge request events`、`Issue events`
1. 確保 **Enable SSL verification** 已勾選（適用於 HTTPS）
1. 點擊 **Add webhook**

### 步驟 3：驗證

使用 GitLab 中的 **Test** 按鈕發送測試事件。確認傳遞成功。

## Gitee Webhook 設定

Gitee（碼雲）webhook 也已受支援。

### 步驟 1：設定環境

Gitee 使用相同的 `WEBHOOK_GITLAB_SECRET` 進行 HMAC 驗證（以權杖作為備援）。或者，如果使用基於密碼的驗證，請設定 `WEBHOOK_GITEE_PASSWORD`。

### 步驟 2：在 Gitee 中建立 Webhook

1. 前往您的倉庫 → **管理** → **Webhooks**
1. 設定 **URL** 為 `https://your-domain.com/api/webhook/gitee`
1. 設定 **Password/Signing Key** 為相同的金鑰
1. 選取事件：`Push`、`Pull Request`、`Issues`
1. 點擊 **Add**

## 自訂 Webhook

`shittim_chest` 在 `/api/webhook/custom/{name}` 支援通用的自訂 webhook 端點。要新增自訂 webhook 來源：

1. 在 `.env` 中設定 `WEBHOOK_PUBLIC_URL`
1. 設定您的外部服務 POST 到 `https://your-domain.com/api/webhook/custom/{name}`
1. 事件將以 webhook 名稱作為事件來源轉發到 scepter

若要在程式碼層級整合新的 webhook 提供者：

1. 在 `packages/core/src/webhook.rs` 中新增處理常式
1. 為新提供者實作 HMAC 或權杖驗證
1. 解析自訂事件格式並透過 Unix socket 轉發到 scepter

## IP 白名單

`shittim_chest` 支援 webhook 來源的 IP 白名單，以拒絕來自未知來源的請求：

```bash
# .env
WEBHOOK_IP_WHITELIST=140.82.112.0/20,192.30.252.0/22  # GitHub IP
```

為每個 webhook 提供者設定 CIDR 範圍。來自白名單外 IP 的請求將被拒絕。

## 事件型別

支援的事件及其到 scepter 觸發器的對應：

| 來源 | 事件 | scepter `event_type` |
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

## 傳遞日誌

`shittim_chest` 維護一個 webhook 事件的傳遞日誌。使用 LRU 快取（最多 10,000 個傳遞 ID）偵測重複傳遞。透過以下方式存取傳遞日誌：

- **REST API**：`GET /api/webhook/deliveries`
- 管理面板：**Webhooks** → **Delivery Log**

## 安全性

所有 webhook 必須通過簽章驗證：

- **GitHub**：使用 `X-Hub-Signature-256` 標頭。依據 `WEBHOOK_GITHUB_SECRET` 進行驗證。
- **GitLab**：使用 `X-Gitlab-Token` 標頭。依據 `WEBHOOK_GITLAB_SECRET` 進行驗證。
- **Gitee**：使用 HMAC-SHA256 簽章，以權杖作為備援。

沒有有效簽章的請求將以 `401 Unauthorized` 拒絕。絕不要在用戶端程式碼或日誌中暴露 webhook 金鑰。

## 測試

使用管理面板測試 webhook 整合：

1. 登入管理面板（預設 `:3000`）
1. 前往側邊欄中的 **Webhooks**
1. 檢視傳遞日誌和設定
1. 透過外部服務的測試功能測試端點

您也可以使用 curl 手動測試：

```bash
curl -X POST https://your-domain.com/api/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=<computed-hmac>" \
  -d '{"action":"push","ref":"refs/heads/main"}'
```

## 疑難排解

### 401 Unauthorized

**原因**：HMAC 簽章不匹配或 IP 不在白名單中。
**修復**：確保 `.env` 中的金鑰與來源平台中設定的金鑰匹配。檢查尾隨空白或編碼問題。驗證 IP 白名單設定。

### 502 Bad Gateway

**原因**：scepter 無法連線。
**修復**：驗證 `.env` 中的 `ENTELECHEIA_SCEPTER_URL` 和 `ENTELECHEIA_TUI_SOCK`。確保 scepter 實例正在執行且 Unix socket 路徑可存取。

### 事件未到達代理

**原因**：事件型別未對應或代理未設定為處理該事件。
**修復**：檢查後端日誌中解析的 `event_type`。驗證目標代理已為該事件註冊處理常式。透過 API 或管理面板檢查傳遞日誌。

### 重複傳遞

**原因**：外部服務因超時而重試。`shittim_chest` 透過 LRU 快取自動偵測重複項目。
**修復**：若有效的重試被封鎖，請增加傳遞 ID 快取大小。確保 `shittim_chest` 在服務的超時時間窗內回應（GitHub：10 秒）。
