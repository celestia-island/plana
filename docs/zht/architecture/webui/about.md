+++
title = "關於 Shittim Chest"
description = """版本 0.1.0"""
lang = "zht"
category = "architecture"
subcategory = "webui"
+++

# Shittim Chest（什亭之匣）

**版本 0.1.0**

Shittim Chest 是 [entelecheia](https://github.com/celestia-island/entelecheia) 多代理協作平台的使用者面向外殼，使用 Rust 和 Vue 3 構建。

## 架構

Shittim Chest 由多個元件組成，共同提供完整的使用者體驗：

- **arona** — 您目前正在使用的聊天 UI，包含串流回應、圖片生成、代理狀態監控、思考視窗、遠端裝置檢視器和多語言支援。
- **`shittim_chest`** — 統一的 Rust + Axum 後端，處理身分驗證（JWT + OAuth）、獨立 LLM 路由、聊天 API、圖片生成、webhook 入口、scepter 代理和遠端裝置信號。

## 與 Entelecheia 的關係

[entelecheia](https://github.com/celestia-island/entelecheia) 是核心的多代理編排引擎。它提供代理執行環境（scepter、13 個專業代理、Cosmos/IEPL 執行環境）。Shittim Chest 處理使用者直接互動的所有內容 — 身分、呈現和通訊。

這兩個專案是透過設計分離的：entelecheia 管理代理編排，而 shittim-chest 管理使用者身分和呈現。它們透過 JWT 驗證的 HTTP/WebSocket 進行通訊。登入憑證儲存在 `shittim_chest_db` 中；權限和身分資料儲存在 `entelecheia_db` 中。這種分離允許前端外殼獨立於代理核心進行演進。

## 與 Hikari 的關係

[hikari](https://github.com/celestia-island/hikari) 是 Celestia Island 生態系統的閘道和路由層。它作為所有外部流量的入口點，處理請求路由、負載均衡和 API 閘道功能，在 shittim-chest、entelecheia 和其他服務之間進行協調。

## 與 Tairitsu 的關係

[tairitsu](https://github.com/celestia-island/tairitsu) 是 Celestia Island 生態系統的跨平台原生應用程式框架。它提供基於 Tauri 的桌面和行動客戶端，將 arona 包裝為原生應用程式，以及支援開發工作流程的瀏覽器自動化和測試基礎設施。

## 授權

Shittim Chest 依據 **Business Source License 1.1 (BSL-1.1)** 授權。

對於**非商業用途** — 包括內部營運、學術研究、教學、個人學習、評估、政府和公共服務以及教育用途 — 授予的權利等同於 **Synthetic Source License 1.0 (SySL-1.0)**（「免費使用授權」）。您可以為這些目的自由使用、研究、修改和執行軟體。

**商業用途** — 例如向第三方提供軟體作為託管服務、將其作為獨立產品重新散佈，或將其用作商業產品的核心元件 — 需要從授權人取得單獨的商業授權。

詳情請參閱[完整授權文字](https://github.com/celestia-island/shittim-chest/blob/main/LICENSE)。

---

由 [Celestia Island](https://github.com/celestia-island) 用 ❤ 構建。
