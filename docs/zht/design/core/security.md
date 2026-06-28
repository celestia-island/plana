+++
title = "Entelecheia 安全架構"
description = """> Entelecheia 多 Agent 編排平台的全面縱深防禦模型。"""
lang = "zht"
category = "design"
subcategory = "core"
+++

# Entelecheia 安全架構

> Entelecheia 多 Agent 編排平台的全面縱深防禦模型。

## 概述

Entelecheia 實作了**縱深防禦安全架構**，涵蓋 14 個可獨立測試的安全層——從硬體級容器隔離到面向 LLM 的工具權限閘道。與直接向 LLM 暴露所有工具的傳統 Agent 框架不同，Entelecheia 的 **Exec-Only 微核心**設計意味著 LLM 僅看到 3 個原語工具（`exec`、`write_to_var`、`write_to_var_json`），而 148 個 MCP 工具透過帶有多層授權的型別化 IEPL 管線進行分派。

## 安全層索引

| # | 層級 | Crate | 緩解的威脅 |
| --- | --- | --- | --- |
| 1 | Exec-Only 微核心 | `scepter`、`mcp_types` | LLM 無限制的工具存取 |
| 2 | 雙重授權權限閘道 | `security_policy` | 未經授權的 MCP 工具調用 |
| 3 | 信任層級技能授權 | `domain_skills_permissions` | 透過技能鏈的特權提升 |
| 4 | 容器隔離（外環） | `container`（Docker/Podman） | 來自 Agent 程式碼的主機入侵 |
| 5 | OCI 沙箱（內環） | `container_runtime`（Youki/libcontainer） | 容器逃逸 |
| 6 | RBAC 存取控制 | `domain_auth`、shittim-chest `rbac` | 未經授權的 API 存取 |
| 7 | JWT 認證 | shittim-chest `auth`（HS256） | 對話劫持、重放攻擊 |
| 8 | API 金鑰加密 | `aporia`（AES-256-GCM） | 靜態憑證洩漏 |
| 9 | 安全哨兵 | `orexis`（OreXis Agent） | 惡意程式碼執行、合規違反 |
| 10 | IEPL 型別安全管線 | `iepl`、`iepl_engine`、`skemma` | 透過未型別化工具調用的注入 |
| 11 | Provider 註冊表白名單 | `config/registries.toml` | 來自不受信任套件的供應鏈攻擊 |
| 12 | 提示注入防禦 | IEPL 沙箱邊界 | 透過工具輸出的 LLM 提示注入 |
| 13 | 速率限制 | shittim-chest `channel/rate_limit` | DoS、資源耗盡 |
| 14 | 審計追蹤 | `orexis`、`timeline` | 事後鑑識、可問責性 |

---

## 第 1 層：Exec-Only 微核心

**Crate：** `scepter`、`mcp_types`
**設計理念：** 最小化 LLM 攻擊面

LLM 在一個**僅 exec 的沙箱**中運作，它只能調用三個原語操作：

| 工具 | 用途 | 參數 |
| --- | --- | --- |
| `exec` | 執行腳本字串 | JavaScript 程式碼（透過 IEPL 從 TypeScript 轉譯） |
| `write_to_var` | 儲存字串值 | 變數名稱 + 值 |
| `write_to_var_json` | 儲存 JSON 值 | 變數名稱 + JSON 值 |

所有 148 個 MCP 工具（檔案操作、容器管理、設備控制、網頁搜尋等）對 LLM **不可見**。當 LLM 的 `exec` 調用 ES 模組導入時（例如 `import { file_read } from 'kalos'`），它們間接地透過 IEPL 管線被調用。

**威脅模型：** 即使 LLM 透過提示注入被入侵，它也無法直接調用危險的工具，如 `container_destroy` 或 `ssh_exec`。IEPL 管線在任何工具執行之前強制執行型別檢查和權限驗證。

**實作：** `packages/shared/mcp_types/src/` 定義微核心 IPC 型別。`packages/cosmos/` 中的 `exec` 處理器透過 Boa 引擎轉譯和執行腳本，工具調用透過 `skemma` 的 `McpRouter` 進行路由。

---

## 第 2 層：雙重授權權限閘道

**Crate：** `security_policy`（5,772 行）

每個 MCP 工具透過**權限層級**列舉宣告其存取需求。每個技能（IEPL 腳本）宣告其對每個工具所需的權限層級。兩者必須一致，調用才能執行。

```rust
pub enum PermissionLevel {
    /// 唯讀操作（file_read、list_dir 等）
    Read,
    /// 工作區內的寫入操作（file_write、exec_script）
    Write,
    /// 影響外部系統的操作（ssh_exec、container_deploy）
    System,
    /// 具有不可逆後果的操作（container_destroy、device_reboot）
    Destructive,
}
```

**授權流程：**

1. 技能宣告："我需要對 `ssh_exec` 的 `System` 存取權"
1. 工具宣告："我需要 `System` 權限"
1. 權限閘道檢查：`skill_level >= tool_requirement` 且 `skill 被明確授予此工具`
1. 若任一檢查失敗：調用被封鎖、記錄並回報給 OreXis 哨兵

**實作：** `packages/shared/security_policy/src/` — 107 個測試註解，4 個 tokio 測試。

---

## 第 3 層：信任層級技能授權

**Crate：** `domain_skills_permissions`（1,776 行）

技能根據決定其預設權限範圍的**信任層級**進行分類：

| 信任層級 | 說明 | 預設權限 |
| --- | --- | --- |
| `Builtin` | 隨平台發布 | 完整工具存取 |
| `Verified` | 由維護者審查和簽署 | 讀寫 |
| `Community` | 使用者提交 | 僅讀取 |
| `Untrusted` | 動態載入 | 無工具存取（僅 exec） |

每個技能的信任層級在載入時驗證並快取。嘗試提升信任層級被記錄為安全事件。

---

## 第 4 層：容器隔離（外環）

**Crate：** `container`（5,742 行）

每個 Agent 執行都在一個 **Docker 或 Podman 容器**內進行，具有：

- 網路命名空間隔離
- 唯讀根檔案系統（工作區掛載除外）
- 限制系統調用的 Seccomp 配置
- 資源限制（CPU、記憶體、PID 數量）
- 無對主機 Docker socket 的存取

**實作：** `packages/shared/container/src/` — 74 個測試註解，12 個 tokio 測試。支援 Docker（透過 Bollard API）和 Podman。

---

## 第 5 層：OCI 沙箱（內環）

**Crate：** `container_runtime`（3,645 行）

在 Docker 容器內部，Entelecheia 使用 Youki/libcontainer 執行**第二層隔離**——一個無守護程序、無 root 的 OCI 相容容器執行時。這提供：

- 無 root 執行（不可能特權提升）
- 獨立於 Docker 的命名空間隔離
- Cgroup v2 強制執行
- Seccomp 過濾器（預設拒絕）

**為什麼需要兩層？** Docker 提供粗粒度的隔離（網路、檔案系統）。Youki 提供細粒度的系統調用過濾和資源記帳。若 Docker 被入侵，Youki 沙箱仍包含該 Agent。

---

## 第 6 層：RBAC 存取控制

**Crate：** `domain_auth`（380 行）、shittim-chest `rbac`（1,736 行）

基於角色的存取控管所有 API 操作：

- **群組：** 使用者屬於群組；群組具有授予
- **授予：** 細粒度權限（按資源類型的讀/寫/管理）
- **工作區隔離：** 使用者只能存取其為成員的工作區
- **跨工作區操作：** 需要顯式管理授予

---

## 第 7 層：JWT 認證

**模組：** shittim-chest `auth/jwt.rs`（264 行）

- **演算法：** HS256（HMAC-SHA256）
- **存取令牌：** 短時效（可配置，預設 15 分鐘）
- **刷新令牌：** 較長時效，使用時輪換
- **基於 Nonce 的 CSRF 保護** 用於瀏覽器客戶端
- **認證端點的速率限制**（GCRA 演算法）

---

## 第 8 層：API 金鑰加密

**Crate：** `aporia`（5,802 行）

所有 LLM 提供者的 API 金鑰使用 **AES-256-GCM** 靜態加密，具有：

- 每次加密操作的唯一 nonce
- 從主機密（環境配置）衍生的金鑰
- 使用後記憶體中明文金鑰的歸零
- 金鑰輪換支援

---

## 第 9 層：安全哨兵（OreXis）

**Crate：** `orexis`（5,239 行） — 「免疫系統」Agent

OreXis 是一個第 1 層 Agent，負責：

- **審計程式碼**以發現安全漏洞和授權合規性
- **檢查工具調用**是否符合已註冊的安全策略
- **封鎖/解除封鎖**任何 Agent 的工具（按模式）
- **監控** Agent 行為以檢測異常模式

MCP 工具（24 個）：`standard_check`、`compliance_report`、`audit_alignment`、`audit_legality`、`agent_integrity`、`security_audit`、`tool_block`、`tool_unblock`、`policy_register`、`policy_list` 等。

---

## 第 10 層：IEPL 型別安全管線

**Crate：** `iepl`（2,670 行）、`iepl_engine`（1,228 行）、`skemma`（7,960 行）

**Entelecheia Plugin Language**（IEPL）管線確保 LLM 生成的程式碼與原生工具分派之間的型別安全：

1. LLM 使用 ES 模組導入生成 TypeScript 程式碼
1. **SWC** 轉譯 TypeScript → JavaScript（語法驗證）
1. **Boa 引擎**在沙箱化上下文中執行 JavaScript
1. ES 模組導入解析為 `__native_dispatch` 調用
1. 每次分派透過 `McpRouter` 進行路由，並進行完整的型別檢查

**緩解的威脅：** 透過未型別化工具調用的注入攻擊（常見於 Python 為基礎的 Agent 框架，其中工具結構僅在執行時驗證）。

---

## 第 11 層：Provider 註冊表白名單

**檔案：** `configs/registries.toml`（337 行）

Entelecheia 維護一個跨 15 個生態系統的受信任套件註冊表的**硬編碼白名單**：

crates.io、PyPI、npm、Go modules、Docker Hub、Maven Central、NuGet、RubyGems、Hackage、Alpine APK、Debian APT、GitHub、GitLab、`HuggingFace`、PyTorch。

來自非白名單註冊表的任何套件導入在執行前在**容器層級被封鎖**。

---

## 第 12 層：提示注入防禦

**機制：** IEPL 沙箱邊界

LLM 的 `exec` 輸出在**隔離的 Boa JS 上下文**中執行，無法存取：

- 主機檔案系統
- 網路 sockets
- 環境變數
- 其他 Agent 的狀態

回傳給 LLM 的工具輸出經過**消毒處理**——二進位資料以 base64 編碼，過多的輸出被截斷，工具結果中潛在的提示注入模式由 OreXis 標記。

---

## 第 13 層：速率限制

**模組：** shittim-chest `channel/rate_limit.rs`（118 行）

使用 **GCRA（Generic Cell Rate Algorithm）** 進行每個使用者、每個通道的速率限制：

- 可配置的突發大小和持續速率
- 每個使用者的 DashMap 以實現 O(1) 查找
- 超過限制時自動退避
- API 調用、訊息發送和工具調用的獨立限制

---

## 第 14 層：審計追蹤

**Crate：** `orexis`、`timeline`（3,096 行）

每個工具調用、Agent 決策和安全事件都會：

1. 記錄在**時間線**中，包含完整上下文（Agent 徽章、技能名稱、參數、結果）
1. 以雜湊鏈結至先前事件，用於篡改檢測
1. 持久化至 PostgreSQL，具有可配置的保留期
1. 可透過 CLI 查詢（`entelecheia-cli trace-chain <badge>`）

---

## 與其他框架的安全比較

| 功能 | Entelecheia | OpenFANG | LangChain | Claude Code |
| --- |  ---  |  ---  |  ---  |  ---  |
| LLM 可見的工具 | **3（僅 exec）** | 53（全部可見） | 全部可見 | 33（全部可見） |
| 容器隔離 | **雙層**（Docker + Youki） | 僅 WASM | 無 | 作業系統層級（Seatbelt/Landlock） |
| 工具權限模型 | **雙重授權** | RBAC | 無 | 無 |
| 程式碼審計 Agent | **OreXis（24 個工具）** | Loop guard | 無 | 無 |
| 型別安全分派 | **IEPL 管線** | 直接函式調用 | 直接函式調用 | 直接函式調用 |
| 套件白名單 | **15 個註冊表** | 無 | 無 | 無 |
| 審計追蹤 | 雜湊鏈結時間線 | Merkle hash-chain | 無 | 無 |

---

## 威脅模型

### 不納入範圍

- 對主機機器的物理存取
- 已入侵的 Docker/Podman 守護程序（假設受信任）
- 核心漏洞（由使用者空間隔離緩解但不防止）
- Rust crate 依賴的供應鏈攻擊（由 `cargo-deny` 部分緩解）

### 接受的風險

- Boa JS 引擎漏洞（在容器內沙箱化）
- LLM 提供者中斷（無備援執行路徑）
- PostgreSQL 資料損壞（由備份緩解，不防止）

---

## 報告漏洞

參見 [SECURITY.md](../SECURITY.md) 以獲取漏洞報告程序。

## 授權

此安全架構是 Entelecheia 的一部分，根據 [BUSL-1.1](../LICENSE) 授權。
