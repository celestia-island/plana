+++
title = "架構深入探討"
description = """> 目標讀者：需要了解 shittim-chest 內部運作方式的開發者。"""
lang = "zht"
category = "guides"
subcategory = "webui"
+++

# 架構深入探討

> **目標讀者**：需要了解 shittim-chest 內部運作方式的開發者。
> **最後更新**：2026-05-25

## 專案總覽

shittim-chest 是 [entelecheia](https://github.com/celestia-island/entelecheia) 的**使用者面向外殼**，後者是一個基於 Rust 的多代理協作平台。邊界是有意設計的：

- **entelecheia** 掌管代理編排（scepter、13 個代理、Cosmos/IEPL 執行環境）、身份和權限。
- **shittim-chest** 掌管使用者身分驗證、會話管理、聊天資料、LLM 提供者設定、前端呈現以及到 scepter 的代理橋接。

它們透過 JWT 驗證的 HTTP 和 WebSocket 進行通訊。shittim-chest 從不直接存取 entelecheia 的資料庫以進行代理操作。

## 後端技術堆疊

### Axum 路由器

核心後端（`packages/core`）是一個 Axum 0.8 應用程式。路由器掛載以下模組群組：

```text
/                   → 健康檢查
/api/auth/*         → AuthService（登入、註冊、GitHub OAuth、刷新、登出）
/api/chat/*         → ChatService（對話、訊息、SSE/WS 串流、搜尋、匯出）
/api/providers/*    → ProviderService（LLM 提供者 CRUD、API 金鑰加密、測試）
/api/generation/*   → GenerationService（圖片生成）
/api/devices/*      → DeviceService（遠端裝置列表、會話、信號）
/api/webhook/*      → WebhookService（GitHub、GitLab、Gitee、自訂；HMAC 驗證）
/api/proxy/*        → ProxyService（HTTP 反向代理 + 到 scepter 的 WebSocket 橋接）
/static/*           → SPA 靜態託管（僅限生產環境）
```

### SeaORM + PostgreSQL

資料庫存取使用 SeaORM 1.x 搭配 PostgreSQL。`shittim_chest_db` 儲存：

- 使用者身分驗證：密碼雜湊（argon2）、會話、刷新權杖、API 金鑰、OAuth 連線
- 聊天資料：對話、訊息
- LLM 提供者設定（API 金鑰使用 AES-256-GCM 靜態加密）
- 遠端裝置記錄和裝置會話
- 多平台訊息傳遞的頻道設定
- Webhook 傳遞日誌

5 個遷移和 25 個實體模型位於 `packages/core/src/{migration,entity}/`。

### JWT 身分驗證

`shittim_chest` 發出包含 `{ sub: user_id, groups: [...] }` 的 JWT。JWT 金鑰與 scepter 共享，因此兩個服務可以獨立驗證權杖。存取權杖在 1 小時後過期；刷新權杖在 7 天後過期，每次使用時輪換。

## 獨立 LLM 能力

shittim-chest 有自己的 LLM 路由層，獨立於 entelecheia 運作：

- **LlmRouter**：具有基於優先級選取和故障轉移的多提供者路由器
- **提供者管理**：用於新增/編輯/移除 LLM 提供者的 CRUD 端點
- **API 金鑰加密**：提供者 API 金鑰使用 AES-256-GCM 靜態加密
- **OpenAI 相容**：可與任何 OpenAI 相容的 API 搭配使用（DeepSeek、OpenAI、本機模型等）
- **雙重串流**：用於聊天回應的 SSE（Server-Sent Events）和 WebSocket 串流

這意味著 shittim-chest 可以作為獨立的聊天應用程式執行，不需要 entelecheia，也可以透過代理層使用 entelecheia 代理。

## 身分驗證流程

### 登入序列

```text
使用者 → shittim_chest：POST /api/auth/login { username, password }
shittim_chest → shittim_chest_db：SELECT user WHERE username = ?（驗證 argon2 雜湊）
shittim_chest → scepter：GET /api/user/{id}/permissions
scepter → entelecheia_db：查詢群組 + 權限
scepter → shittim_chest：{ groups: [...], permissions: {...} }
shittim_chest → 使用者：{ access_token, refresh_token }
shittim_chest：儲存會話 + 快取 RBAC
```

### GitHub OAuth

```text
使用者 → shittim_chest：GET /api/auth/github
shittim_chest → 使用者：302 重定向到 GitHub OAuth
使用者 → GitHub：授權
GitHub → shittim_chest：GET /api/auth/github/callback?code=...
shittim_chest → GitHub：交換 code 換取 access token
shittim_chest → GitHub：GET /user（取得使用者資訊）
shittim_chest → shittim_chest_db：INSERT/UPDATE oauth_connections
shittim_chest → 使用者：{ access_token, refresh_token }（新使用者自動建立帳號）
```

## 聊天架構

### 訊息流程（獨立 LLM）

```text
使用者 → POST /api/chat/conversations/:id/messages
shittim_chest：驗證 JWT，載入對話
shittim_chest → LlmRouter：將請求路由到最佳提供者
LlmRouter → LLM 提供者：POST chat/completions（串流）
LLM 提供者 → LlmRouter：SSE 串流
LlmRouter → 使用者：SSE/WS 串流（權杖即時送達）
shittim_chest：將訊息持久化到 shittim_chest_db
```

### SSE vs WebSocket 串流

- **SSE**（`/api/chat/stream`）：簡單的 HTTP 串流，可透過代理運作，自動重新連線
- **WebSocket**（`/ws/chat/stream`）：雙向，支援取消和即時互動

## 代理架構

`/api/proxy/*` 端點將已驗證的請求轉發到 scepter：

1. 瀏覽器以 JWT 開啟 `ws://shittim-chest:80/api/proxy/chat`
1. `shittim_chest` 驗證 JWT，開啟到 scepter 的連線並轉發 JWT
1. 瀏覽器和 scepter 之間的雙向訊息轉發
1. `shittim_chest` 強制執行速率限制、記錄使用情況、管理連線生命週期

## Webhook 管線

來自外部服務的 Webhook 透過 `/api/webhook/*` 進入：

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC 驗證 → 解析事件 → 透過 Unix socket 轉發到 scepter
```

支援的來源：GitHub（HMAC-SHA256）、GitLab（權杖）、Gitee（HMAC + 權杖備援），加上一個通用的 `/api/webhook/custom/{name}` 端點。功能：

- 重複傳遞偵測（LRU 快取，10,000 個 ID）
- 帶有列表 API 的傳遞日誌
- webhook 來源的 IP 白名單

## 遠端裝置管理

遠端裝置透過信號中繼管理：

```text
瀏覽器 (webui) → WS /api/devices/stream → shittim_chest（信號中繼）→ Unix socket → entelecheia/polemos
```

功能：

- 透過 REST 的裝置列表和會話 CRUD
- WebRTC 信號（SDP offer/answer、ICE 候選）
- 終端中繼（WebSocket 到 xterm.js）
- 桌面影格轉發
- SFTP 檔案瀏覽器後端

shittim-chest 從不直接連線到遠端裝置 — 所有資料流都透過 entelecheia 的 polemos 代理。

## 資料所有權

### shittim_chest_db

| 資料 | 資料表 | 理由 |
| --- | --- | --- |
| 密碼雜湊（argon2） | `auth_users` | 呈現層掌管登入流程 |
| 活動會話、刷新權杖 | `sessions` | 會話管理是前端關注事項 |
| 加密的 API 金鑰 | `api_keys` | API 金鑰發放是使用者面向的 |
| OAuth 連線 | `oauth_connections` | 第三方驗證繫結是使用者面向的 |
| 對話、訊息 | `conversations`、`messages` | 聊天資料是使用者面向的 |
| LLM 提供者設定 | `llm_providers` | 提供者管理是使用者面向的（金鑰已加密） |
| 遠端裝置記錄 | `remote_devices`、`device_sessions` | 裝置追蹤是使用者面向的 |
| 頻道設定 | `channel_configs` 等 | 多平台設定是使用者面向的 |

### entelecheia_db

| 資料 | 理由 |
| --- | --- |
| 使用者身份、群組、角色指派 | 核心強制執行權限 |
| GroupPermissions（提供者配額、代理白名單） | 代理層級政策與代理同在 |
| 代理設定、Cosmos/IEPL 狀態 | 編排資料屬於核心 |

## 雙重前端策略

### 第一階段：Vue 3（目前）

| 套件 | 技術 | 連接埠 | 用途 |
| --- | --- | --- | --- |
| `webui` | Vue 3 + Vite + Pinia (TSX) | `:3000 (共享)` | 統一 webui：聊天、圖片生成、裝置、管理（提供者、代理、RBAC、webhook） |

### 第二階段：Rust WASM（未來）

| 套件 | 技術 | 用途 |
| --- | --- | --- |
| `webui` | Rust → WASM (Tairitsu) | 長期統一 webui（聊天 + 管理） |

舊版前端作為活的規格。在過渡期間，兩個版本並行執行，相同的使用者互動必須產生相同的結果。

## 反向代理部署模式

shittim-chest 支援三種反向代理模式，由 `.env` 中的 `SHITTIM_CHEST_PROXY_MODE` 控制。

### 模式 1：無（直接）

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=none   # 或未設定
```

核心伺服器直接繫結到 `SHITTIM_CHEST_HOST:SHITTIM_CHEST_PORT`（預設 `0.0.0.0:80`）。無 TLS，無反向代理容器。適用於：

- 本機開發
- 在現有反向代理後方（Cloudflare Tunnel、AWS ALB、Traefik labels）
- 其他服務處理 TLS 終止的 Docker 網路

### 模式 2：Caddy 自動

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_DOMAIN=app.example.com
```

CLI 建立一個 `shittim-chest-caddy` 容器（映像 `caddy:2`），該容器：

1. 監聽連接埠 80/443（可透過 `SHITTIM_CHEST_PROXY_HTTP_PORT` / `SHITTIM_CHEST_PROXY_HTTPS_PORT` 設定）
1. 透過 Let's Encrypt（Caddy 內建的 ACME）自動佈建 TLS 憑證
1. 將所有請求代理到 Docker 網路上的核心後端

不需要 Caddyfile — CLI 會自動生成一個。網域必須有指向主機的公開 DNS。

### 模式 3：Caddy 自訂

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/caddy/Caddyfile
SHITTIM_CHEST_PROXY_EXTRA_VOLUMES=/etc/letsencrypt:/etc/letsencrypt
```

相同的 Caddy 容器，但您提供自己的 Caddyfile（從主機掛載）。當您需要以下功能時使用此模式：

- 多個虛擬主機
- 自訂 TLS 憑證路徑
- 額外的中介軟體（基本驗證、速率限制等）
- 與 API 一起提供靜態檔案

### 模式 4：Nginx 自訂

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=nginx
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/nginx/conf.d/default.conf
```

使用您的設定檔建立一個 `nginx:bookworm` 容器。您自行管理 TLS 憑證。適用於以 Nginx 為標準的環境。

### 容器生命週期

所有代理容器由 CLI 透過 Docker API（`bollard`）管理：

| 命令 | 行為 |
| --- | --- |
| `just dev` / `chest up` | 若設定了 `PROXY_MODE`，則建立/啟動代理容器 |
| `just dev-stop` / `chest down` | 停止並移除代理容器 |
| 容器已在執行 | 重複使用現有容器（冪等） |

代理容器加入與核心後端相同的 Docker 網路，因此它透過內部主機名稱（`core` 或 `shittim-chest`）連線到後端。
