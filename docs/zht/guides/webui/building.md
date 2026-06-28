+++
title = "構建與開發指南"
description = """> 目標讀者：正在設定本機 shittim-chest 開發環境的貢獻者。"""
lang = "zht"
category = "guides"
subcategory = "webui"
+++

# 構建與開發指南

> **目標讀者**：正在設定本機 shittim-chest 開發環境的貢獻者。
> **最後更新**：2026-05-25

## 先決條件

| 工具 | 最低版本 | 備註 |
| --- | --- | --- |
| Rust | 1.85+ | 需要 Edition 2024；透過 <https://rustup.rs> 安裝 |
| Node.js | 20+ | 建議使用 LTS |
| pnpm | 9+ | `corepack enable && corepack prepare pnpm@latest --activate` |
| just | 最新版 | 命令執行器；`cargo install just` |
| PostgreSQL | 18+ | 用於 auth + 聊天資料的 shittim_chest_db |
| entelecheia scepter | 可選 | 代理/裝置功能需要；獨立聊天可選 |

驗證一切：

```bash
rustc --version    # >= 1.85
node --version     # >= 20
pnpm --version     # >= 9
just --version
psql --version     # >= 18
```

## 克隆與引導

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
```

## 環境變數

克隆後編輯 `.env`。每個變數都在檔案內有文件說明；以下是摘要。

### 伺服器

| 變數 | 預設值 | 用途 |
| --- | --- | --- |
| `SHITTIM_CHEST_HOST` | `0.0.0.0` | 監聽地址 |
| `SHITTIM_CHEST_PORT` | `80` | 監聽連接埠 |

### 資料庫

| 變數 | 預設值 | 用途 |
| --- | --- | --- |
| `SHITTIM_CHEST_DATABASE_URL` | `postgresql://sc:pass@localhost:5432/shittim_chest` | PostgreSQL 連線字串 |
| `SHITTIM_CHEST_DATABASE_MAX_CONNECTIONS` | `10` | SeaORM 連線池大小 |

建立資料庫和使用者：

```sql
CREATE USER sc WITH PASSWORD 'pass';
CREATE DATABASE shittim_chest OWNER sc;
```

### JWT 與加密

| 變數 | 預設值 | 用途 |
| --- | --- | --- |
| `JWT_SECRET` | `change-me-in-production` | 與 scepter 共享的金鑰；**必須匹配** |
| `JWT_EXPIRATION_SECONDS` | `3600` | 存取權杖生命週期（1 小時） |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | 刷新權杖生命週期（7 天） |
| `SHITTIM_CHEST_ENCRYPTION_KEY` | `change-me-32-bytes-base64-encoded` | 用於 API 金鑰和 OAuth 權杖的 AES-256-GCM 金鑰 |

生成一個生產用金鑰：

```bash
openssl rand -base64 32
```

### LLM 提供者（用於獨立操作）

設定這些以獨立使用 shittim-chest 而不需要 entelecheia：

| 變數 | 用途 |
| --- | --- |
| `LLM_DEFAULT_PROVIDER_ENDPOINT` | OpenAI 相容的 API 端點（例如 `https://api.deepseek.com/v1`） |
| `LLM_DEFAULT_PROVIDER_API_KEY` | 提供者的 API 金鑰 |
| `LLM_DEFAULT_PROVIDER_MODELS` | 逗號分隔的模型列表（例如 `deepseek-chat,deepseek-reasoner`） |
| `LLM_DEFAULT_PROVIDER_CATEGORY` | 提供者類別：`chat` 或 `image` |
| `LLM_STREAM_BUFFER_SECONDS` | 串流緩衝超時（預設：60） |
| `LLM_MAX_TOKENS_DEFAULT` | 預設最大權杖數（預設：4096） |
| `LLM_REQUEST_TIMEOUT_SECONDS` | HTTP 請求超時（預設：120） |

### 遠端裝置

| 變數 | 預設值 | 用途 |
| --- | --- | --- |
| `REMOTE_DEVICES_ENABLED` | `false` | 啟用遠端裝置功能 |
| `REMOTE_DEVICES_SCEPTER_SOCK` | `/run/entelecheia/device_stream.sock` | 用於裝置資料的 Unix socket |
| `REMOTE_DEVICES_FRAME_BUFFER_SIZE` | `4194304` | 影格緩衝區大小（位元組） |
| `REMOTE_DEVICES_MAX_SESSIONS_PER_USER` | `3` | 最大並發裝置會話數 |
| `WEBRTC_ICE_SERVERS` | `stun:stun.l.google.com:19302` | ICE 伺服器列表 |

### GitHub OAuth

| 變數 | 用途 |
| --- | --- |
| `GITHUB_CLIENT_ID` | GitHub OAuth App 用戶端 ID |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth App 用戶端密鑰 |
| `GITHUB_REDIRECT_URI` | OAuth 回呼 URL（例如 `https://your-domain/api/auth/github/callback`） |

### Scepter 連線（用於代理功能）

| 變數 | 預設值 | 用途 |
| --- | --- | --- |
| `ENTELECHEIA_SCEPTER_URL` | `http://localhost:8424` | scepter 的 HTTP 端點 |
| `ENTELECHEIA_SCEPTER_WS_URL` | `ws://localhost:8424` | scepter 的 WebSocket 端點 |
| `ENTELECHEIA_TUI_SOCK` | `/run/entelecheia/entelecheia.sock` | 用於觸發轉發的 Unix socket |

### Webhook

| 變數 | 用途 |
| --- | --- |
| `WEBHOOK_GITHUB_SECRET` | 用於 GitHub webhook 驗證的 HMAC 金鑰 |
| `WEBHOOK_GITLAB_SECRET` | 用於 GitLab webhook 驗證的權杖 |
| `WEBHOOK_PUBLIC_URL` | webhook 端點的公開面向 URL |

## 資料庫設定

```bash
just db-init      # 建立結構描述（執行 SeaORM 遷移）
just db-migrate   # 套用待處理的遷移
```

### 結構描述總覽

`shittim_chest_db` 掌管使用者面向的資料：

| 資料表 | 用途 |
| --- | --- |
| `auth_users` | 帶有 argon2 密碼雜湊的使用者帳號 |
| `sessions` | 帶有刷新權杖的活動會話 |
| `api_keys` | API 金鑰記錄（已雜湊） |
| `oauth_connections` | 第三方 OAuth 繫結（GitHub） |
| `conversations` | 聊天對話 |
| `messages` | 帶有工具呼叫資料的聊天訊息 |
| `llm_providers` | LLM 提供者設定（API 金鑰已加密） |
| `remote_devices` | 遠端裝置記錄 |
| `device_sessions` | 活動裝置會話 |
| `channel_configs` | 多平台頻道設定 |
| `channel_messages` | 頻道訊息記錄 |
| `channel_pairings` | 頻道到聊天的配對 |

重設資料庫：

```bash
just db-reset
```

## 後端開發

```bash
just dev-backend
```

這會執行 `cargo run --package shittim_chest`。伺服器在 `:80` 上啟動。

### CLI 命令

```bash
shittim_chest db-init      # 建立資料庫結構描述
shittim_chest db-migrate   # 套用待處理的遷移
shittim_chest db-reset     # 刪除並重新建立結構描述
shittim_chest server       # 啟動網頁伺服器
```

### 熱重載

```bash
cargo install cargo-watch
cargo watch -x 'run --package shittim_chest -- server'
```

### API 端點總覽

| 路由群組 | 用途 |
| --- | --- |
| `/api/auth/*` | 登入、註冊、GitHub OAuth、刷新、登出 |
| `/api/chat/*` | 對話、訊息、SSE/WS 串流、搜尋、匯出 |
| `/api/providers/*` | LLM 提供者 CRUD、API 金鑰管理、測試 |
| `/api/generation/*` | 圖片生成、模型列表 |
| `/api/devices/*` | 遠端裝置列表、會話、WebRTC 信號 |
| `/api/webhook/*` | GitHub/GitLab/Gitee/自訂 webhook 入口 |
| `/api/proxy/*` | 到 scepter 的反向代理（HTTP + WebSocket） |
| `/static/*` | SPA 靜態檔案託管 |

## 前端開發

### 安裝依賴

```bash
pnpm install
```

### webui

```bash
just dev    # 構建前端 + 在 :3000 上啟動後端
just watch  # 檔案變更時自動重建
```

兩個前端都由 Vite 構建到 `dist/`。後端直接在 `:3000` 上提供這些靜態檔案 — 不需要單獨的 Vite 開發伺服器或代理。在開發模式下，`dev.py` 會監視前端原始碼並自動重建。

## 跨專案設定

要使用共享的 `arona` 協定 crate 進行本機開發，請將其修補到您的本機 checkout。編輯 `~/.cargo/config.toml`（永不提交到倉庫）：

```toml
[patch.'https://github.com/celestia-island/arona']
arona = { path = "/path/to/arona" }
```

對於 npm，webui 透過 `@celestia-island/arona` 路徑別名消費 `arona` crate 的 TS 繫結，指向 `packages/webui/src/types/arona/`。

## 為生產構建

```bash
just build
```

這會執行 `cargo build --release` 和 `pnpm run build:all`。輸出位置：

- 後端二進位檔：`target/release/shittim_chest`
- 前端資源：`packages/webui/dist/`

### Docker

使用 CLI 包裝器構建和執行（直接使用 Docker API）：

```bash
just dev
```

或手動執行：

```bash
just build        # 構建 Docker 映像
just up           # 啟動所有服務
just migrate      # 執行資料庫遷移
```

生產二進位檔透過 Axum 的靜態檔案中介軟體在 `/` 提供前端資源。不需要單獨的前端伺服器。

## 常見問題

### 資料庫連線被拒絕

```text
error: connection to server at "localhost", port 5432 failed
```

**修復**：確保 PostgreSQL 正在執行，且 `.env` 中的 `SHITTIM_CHEST_DATABASE_URL` 與您的設定匹配。使用 `psql "$SHITTIM_CHEST_DATABASE_URL" -c 'SELECT 1'` 驗證。

### Scepter 無法連線

```text
error: error sending request for url (http://localhost:8424/...)
```

**修復**：啟動 entelecheia scepter 實例，或使用已設定 LLM 提供者的獨立模式。後端在無 scepter 的情況下仍可進行聊天/圖片生成。

### 瀏覽器中的 CORS 錯誤

```text
Access-Control-Allow-Origin header is present on the requested resource
```

**修復**：開發後端為 `localhost` 來源啟用 CORS。如果您變更了連接埠，請更新 CORS 設定。生產部署應設定反向代理（nginx/caddy）來處理 CORS。

### pnpm install 失敗

**修復**：確保您使用的是 pnpm 9+。執行 `corepack enable && corepack prepare pnpm@latest --activate` 以設定正確的版本。

### 共享 crate 的 cargo build 失敗

**修復**：如果您在 `~/.cargo/config.toml` 中有本機補丁，請確保路徑存在且 crate 名稱匹配。移除補丁區段以改用 git 依賴。
