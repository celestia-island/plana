+++
title = "ADR-004：60+ Crate 分層工作區架構"
description = """日期：2026-03"""
lang = "zht"
category = "design"
subcategory = "core"
+++

# ADR-004：60+ Crate 分層工作區架構

**日期**：2026-03
**狀態**：已接受

## 背景

Entelecheia 始於一個單體式的 `packages/shared` crate（38K 行，187 個 `.rs` 檔案），包含所有共享基礎設施：型別、MCP 協定、LLM 提供者、容器管理、資料庫、安全、配置等。隨著專案成長為 12 個 Agent + 1 個領域 Agent + 3 個二進位套件，若干問題浮現：

1. **編譯時間**：對 `shared` 的任何修改都需要重新編譯全部 187 個檔案，即使只修改了一個結構體。
1. **依賴污染**：僅需要 MCP 型別的 Agent crate 被迫傳遞依賴於資料庫驅動程式、容器執行時和 LLM 提供者。
1. **所有權不明確**：一個 crate 中有 187 個檔案，不清楚哪個模組「擁有」哪個功能，使重構充滿風險。
1. **功能標誌爆炸**：使用 Cargo feature 進行條件編譯以避免引入不必要的依賴，但導致測試配置的組合爆炸。

## 決策

將單體式的 `packages/shared` 分解為 **37 個專注的子 crate**，組織於 **6 個依賴層**（L0 到 L5），遵循嚴格的依賴方向：

```text
L0（葉節點）→ L1 → L2 → L3 → L4 → L5 → 消費者（scepter、agents、tui）
```

**層級定義：**

| 層級 | Crate | 規則 |
| --- | --- | --- |
| **L0** | core、logging、macros | 對其他 entelecheia crate 零內部依賴 |
| **L1** | domain_enums、mcp_types、text、concurrent | 僅依賴 L0 |
| **L2** | config、agent_registry、state_types | 依賴 L0-L1 |
| **L3** | domain_agent、container、agent_lifecycle、agent_runtime、thread_types、toolchain、infra_utils | 依賴 L0-L2 |
| **L4** | state_sync、domain_skills、hooks、domain_auth、container_runtime、skills_permissions、timeline、iepl | 依賴 L0-L3 |
| **L5** | llm_provider、prompt、custom_agent、storage、infra_jsonrpc、infra_services、e2e_events、adapter、plugin_host、rag、embedding、security_policy | 依賴 L0-L4 |

所有內部依賴宣告使用 `workspace = true` 以確保版本一致性。不存在薄的聚合 crate——消費者直接從單個子 crate 導入。

## 後果

### 正面

- **增量編譯**：對 `shared-core`（L0）的修改仍會傳播，但對 `shared-security-policy`（L5）的修改只會重新編譯該 crate 及其直接消費者。建置時間顯著改善。
- **明確的所有權邊界**：每個 crate 有專注的職責。程式碼審查範圍自然受 crate 邊界約束。
- **依賴隔離**：Agent crate 僅導入其所需的共享 crate。SkeMma 不會拉入資料庫驅動程式。EleOs 不會拉入容器執行時。
- **循環依賴預防**：分層架構使循環依賴在結構上不可能——L3 crate 無法依賴 L5 crate。
- **可獨立測試**：每個 crate 的測試獨立執行，不需要完整工作區的依賴樹。

### 負面

- **工作區管理開銷**：單一工作區中有 60+ crate 意味著更多 `Cargo.toml` 檔案需要維護，更多 `[dependencies]` 段落需要在版本升級時更新，以及更謹慎的依賴宣告。
- **跨 crate 重構更困難**：將型別從 L2 移到 L3 需要更新所有 L2 消費者，並驗證沒有 L3+ crate 意外透過舊位置依賴於移動的型別。
- **Crate 命名冗長**：內部 crate 名稱使用 `_shared_*` 前綴慣例（如 `_shared_domain_skills_permissions`），雖然冗長，但對工作區清晰度是必要的。
- **潛在的過度分解**：某些 crate（如 `shared-text`，約 200 行）可能不值得其獨立的 crate 開銷。分解遵循「若可能增長則分離」的哲學，而非嚴格必要性。

### 接受的權衡

**管理複雜性換取編譯時間與架構清晰度。** 將 `shared` 分解為 37 個 crate 處於 Rust 工作區設計的激進端。折中方案（10-15 個 crate）在管理上會更簡單。然而，鑑於專案的廣泛覆蓋面（26 個 LLM 提供者、2 個容器執行時、12 個 Agent、完整安全管線、資料庫、IEPL），精細分解確保每個部分都可獨立演進。`workspace = true` 模式減輕了版本管理開銷。
