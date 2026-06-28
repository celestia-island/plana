+++
title = "內嵌前端策略"
description = """shittim-chest 支援兩種前端託管模式：在開發模式下，`dev.py` 監視前端原始碼並在變更時觸發 `pnpm build`，後端在 `:3000` 埠上同時提供靜態檔案和 API；在發布模式下，前端靜態檔案在編譯時嵌入到 Rust 二進位檔中，並在 `:80` 上提供服務。兩種模式透過 `embedded-frontend` Cargo 功能切換，使用 `#[cfg(feature = "embedded-frontend")]` 進行程式碼層級的條件編譯。"""
lang = "zht"
category = "design"
subcategory = "webui"
+++

# 內嵌前端策略

## 概述

shittim-chest 支援兩種前端託管模式：在開發模式下，`dev.py` 監視前端原始碼並在變更時觸發 `pnpm build`，後端在 `:3000` 埠上同時提供靜態檔案和 API；在發布模式下，前端靜態檔案在編譯時嵌入到 Rust 二進位檔中，並在 `:80` 上提供服務。兩種模式透過 `embedded-frontend` Cargo 功能切換，使用 `#[cfg(feature = "embedded-frontend")]` 進行程式碼層級的條件編譯。

## 架構比較

```mermaid
flowchart TB
    subgraph Dev[開發模式：dev.py + 後端]
        D1[dev.py 監視前端原始碼] --> D2[pnpm build → dist/]
        D2 --> D3[shittim_chest :3000 提供靜態 + API 服務]
    end
    subgraph Release[發布模式：內嵌]
        R1[瀏覽器] --> R2[shittim_chest :80]
        R2 --> R3[API + LLM]
        R2 --> R4[/static/*\n內嵌 SPA]
    end
```

| 維度 | 開發模式（無功能） | 發布模式（embedded-frontend） |
| --- | --- | --- |
| 前端原始碼 | 由 Vite 建置，後端提供服務 | `include_dir!` 編譯時嵌入 |
| 熱重載 | 透過 dev.py 自動重建 | 不支援（靜態） |
| API 請求路由 | 瀏覽器直接連線（同源） | 瀏覽器直接連線 |
| 二進位檔大小 | 僅後端 | + 前端 dist/ 目錄 |
| 需要 Node | 是（僅建置時） | 否 |
| 啟動方式 | `dev.py`（監視 + 重建） | `just up` 一次性啟動 |

## 實作細節

### 條件編譯

```rust
# [cfg(feature = "embedded-frontend")]
static ARONA_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../dist/arona");

async fn serve_arona() -> impl IntoResponse {
    #[cfg(feature = "embedded-frontend")]
    {
        // 從編譯時嵌入的 Dir 讀取
    }
    #[cfg(not(feature = "embedded-frontend"))]
    {
        // 從檔案系統 ./dist/arona/index.html 讀取
    }
}
```

條件編譯在**函式主體層級**操作，而非模組層級，保持兩種模式下的公開 API 完全相同。

### SPA 回退

應用程式為單頁應用。所有不匹配靜態資源的路由皆回傳 `index.html`：

```text
GET /               → index.html
GET /chat/123       → index.html（前端路由器處理）
GET /backend        → index.html
GET /backend/providers → index.html（前端路由器處理）
```

### MIME 類型偵測

靜態檔案服務根據副檔名回傳正確的 Content-Type：

| 副檔名 | Content-Type |
| --- | --- |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.html` | `text/html` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff/.woff2` | `font/woff2` |
| 其他 | `application/octet-stream` |

## Dockerfile 中的前端建置

```text
階段 1（前端）：
  node:22-slim → pnpm install → pnpm build:all → /app/dist/arona/

階段 2（建置器）：
  rust:1.85-slim → COPY /app/dist/ → cargo build --features embedded-frontend

階段 3（執行時期）：
  debian:bookworm-slim → COPY 二進位檔 → ENTRYPOINT ["./shittim_chest"]
```

前端建置和 Rust 編譯在同一個 Dockerfile 內完成。最終執行時期映像僅包含編譯後的二進位檔。

## 設計決策

1. **開發模式使用 dev.py 進行自動重建**：`dev.py` 監視前端原始碼並在變更時重建，後端在單一埠上提供所有服務。
1. **發布模式無需反向代理**：二進位檔內嵌 SPA，實現單一程序部署，降低運維複雜度。
1. **前端不在執行時期動態載入**：避免檔案系統依賴和版本不一致。發布映像僅包含單一二進位檔案。
1. **單一 SPA**：前端在 `/` 提供服務，管理面板在 `/backend`。
