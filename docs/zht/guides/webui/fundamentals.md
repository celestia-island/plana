+++
title = "核心概念"
description = """> 目標讀者：希望從概念上理解 shittim-chest 設計的開發者。"""
lang = "zht"
category = "guides"
subcategory = "webui"
+++

# 核心概念

> **目標讀者**：希望從概念上理解 shittim-chest 設計的開發者。
> **最後更新**：2026-05-25

## 雙倉庫架構

shittim-chest 和 [entelecheia](https://github.com/celestia-island/entelecheia) 構成一個雙倉庫系統，具有明確的邊界：

- **entelecheia** — 代理編排核心（scepter、13 個代理、Cosmos/IEPL 執行環境）。掌管身份、權限、代理設定。
- **shittim-chest** — 使用者面向外殼。掌管身分驗證、會話、聊天資料、LLM 提供者設定、前端 UI 和到 scepter 的代理橋接。

它們透過 JWT 驗證的 HTTP 和 WebSocket 進行通訊。兩者都不直接存取對方的資料庫。這種分離允許每個倉庫獨立開發、部署和擴展。

## 雙重運作模式

shittim-chest 支援兩種運作模式：

### 獨立模式

使用自己的 LLM 路由層獨立執行。支援：

- 帶有串流回應的聊天（SSE + WebSocket）
- 透過已設定的提供者進行圖片生成
- 使用者身分驗證（密碼 + GitHub OAuth）
- 提供者管理（新增/移除 LLM 提供者）

不需要 entelecheia。適合開發和簡單部署。

### 代理模式

作為 entelecheia 代理系統的閘道。新增：

- 帶有 JWT 傳遞的請求轉發到 scepter
- 用於基於代理聊天的 WebSocket 橋接
- Webhook 入口和觸發轉發
- 透過 polemos 的遠端裝置管理
- RBAC 權限查詢和快取

需要執行中的 entelecheia 實例。兩種模式可以共存 — 獨立 LLM 用於簡單聊天，代理用於代理編排。

## 身分驗證模型

身分驗證使用由 `shittim_chest` 發出的 JWT 權杖：

1. **憑證儲存**：密碼（argon2 雜湊）、會話、刷新權杖和 API 金鑰儲存在 `shittim_chest_db` 中。
1. **GitHub OAuth**：使用者可以使用 GitHub 登入；帳號在首次登入時自動建立。
1. **權限儲存**：使用者群組、角色和權限矩陣儲存在 `entelecheia_db` 中。
1. **JWT 流程**：登入時，`shittim_chest` 在本機驗證憑證，然後從 scepter 取得權限。發出的 JWT 包含 `{ sub: user_id, groups: [...] }`。
1. **共享金鑰**：JWT 簽署金鑰與 scepter 共享，以便兩個服務可以獨立驗證權杖。
1. **權杖輪換**：存取權杖在 1 小時後過期；刷新權杖在 7 天後過期。刷新權杖在每次使用時輪換。

## 前端（webui）

webui 是位於 `packages/webui/` 的統一前端，聊天介面在 `/`，管理面板在 `/backend`，使用 Vue 3 + Vite + Pinia 構建（TSX 透過 `@vitejs/plugin-vue-jsx`）。

## LLM 提供者系統

shittim-chest 具有獨立的 LLM 路由層：

- **提供者**：可設定的 LLM API 端點（OpenAI 相容）。以 AES-256-GCM 加密的 API 金鑰儲存在 `shittim_chest_db` 中。
- **路由器**：具有基於優先級選取和自動故障轉移的多提供者路由。
- **類別**：提供者可以標記為 `chat`、`image` 或兩者。
- **管理**：透過 REST API 和 webui 管理面板進行完整的 CRUD。提供者可以測試連通性。
- **串流**：同時支援 SSE（簡單、代理友好）和 WebSocket（雙向）串流協定。

## 聊天系統

- **對話**：具有標題和中繼資料的基於討論串的聊天會話
- **訊息**：支援文字、圖片和工具呼叫（函數呼叫）
- **串流**：透過 SSE 或 WebSocket 的即時逐個權杖回應傳遞
- **搜尋**：使用 ILIKE 查詢的全文訊息搜尋
- **匯出**：對話可以匯出為 JSON 或 Markdown 格式
- **圖片生成**：透過已設定提供者的基於提示詞的圖片生成，具有「插入到聊天」功能

## 遠端裝置管理

shittim-chest 為由 entelecheia/polemos 管理的遠端裝置提供基於瀏覽器的介面：

- **桌面**：帶有影格轉發的基於 WebRTC 的遠端桌面檢視器
- **終端**：帶有 WebSocket 中繼的基於 xterm.js 的終端模擬器
- **檔案瀏覽器**：SFTP 檔案瀏覽器後端（骨架）
- **信號**：基於 WebSocket 的 WebRTC 信號中繼（SDP offer/answer、ICE 候選）

所有裝置通訊都透過 entelecheia 的 polemos 代理進行 — shittim-chest 從不直接連線到端點。

## 代理架構

`shittim_chest` 作為使用者和 scepter 之間的閘道：

- **HTTP 反向代理**：`/api/proxy/*` 將已驗證的請求透過 JWT 傳遞轉發到 scepter。
- **WebSocket 橋接**：聊天串流使用雙向 WebSocket 轉發（`瀏覽器 ↔ shittim_chest ↔ scepter`）。

這允許 `shittim_chest` 強制執行速率限制、記錄使用情況和管理連線生命週期，而無需 scepter 處理各個瀏覽器連線。

## Webhook 管線

外部事件透過 webhook 管線到達代理核心：

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC 驗證 → 解析事件 → 透過 Unix socket 轉發到 scepter → 代理分派
```

每個提供者都有自己的驗證機制：

- **GitHub**：透過 `X-Hub-Signature-256` 的 HMAC-SHA256
- **GitLab**：透過 `X-Gitlab-Token` 的權杖
- **Gitee**：帶有權杖備援的 HMAC

其他功能：重複傳遞偵測（LRU 快取）、傳遞日誌記錄、IP 白名單和通用的自訂 webhook 端點。

## RBAC 模型

權限遵循基於群組的 RBAC 模型：

- **群組**：使用者屬於一個或多個群組。
- **角色**：群組具有指派的角色。
- **權限**：每個角色定義一個權限矩陣，涵蓋：
  - 提供者配額（最大權杖數、最大請求數）
  - 代理白名單（群組可以存取哪些代理）
  - 管理能力（管理使用者、設定提供者）

`shittim_chest` 在處理程序中快取權限，具有 TTL（預設 5 分鐘）。快取失效發生在 TTL 過期、登出或從 scepter 傳播的明確權限變更時。

## 前端策略

shittim-chest 使用兩階段前端方法：

**第一階段（目前）**：Vue 3 前端（`webui`，位於 `packages/webui/`），使用 Vite + Pinia 構建，透過 `@vitejs/plugin-vue-jsx` 使用 TSX。它定義了 API 合約，並作為生產品質的參考實作。

**第二階段（未來）**：使用 Tairitsu 構建的 Rust → WASM 前端。舊版前端作為活的規格和測試神諭 — 相同的使用者互動必須產生相同的結果。

## 型別安全橋接

TypeScript 型別是透過外部的 `arona` 協定 crate 從 Rust 程式碼生成的，確保前後端一致性：

```text
arona Rust crate（git 依賴）
  → #[derive(ts_rs::TS)]
  → ts-rs 程式碼生成 → packages/webui/src/types/arona/（TypeScript）
  → 由 webui 以 @celestia-island/arona 消費
```

這消除了手動型別同步。當 `arona` crate 中的 Rust 型別變更時，TypeScript 繫結會重新生成並由 webui 使用。
