+++
title = "工業走廊整合計畫"
description = """> 目標：系統必須展示對一個完全未知的工業示範走廊的自主自我介接"""
lang = "zht"
category = "architecture"
subcategory = "core"
+++

# 工業走廊自主整合計畫

> **目標**：系統必須展示對一個完全未知的工業示範走廊的**自主自我介接** — 發現硬體、推斷資料模型、生成監控設定、並關閉警報→回應迴路 — 無需手動的每裝置工程。
> **政府強制截止日期**：此能力與政府專案里程碑綁定。

---

## 剩餘工作

完整的發現 → 推斷 → 監控 → 警報 → **寫入審批**鏈已交付
（階段 A.1–A.3、B、C、D.1、**D.2 ✓**）。唯一剩餘的工作是
**端對端 dogfood 驗證（階段 E）** — 操作性的，而非程式碼。

### D.2 — 寫入審批往返（環內人工）✓

```text
代理決定需要寫入
  → verify_write_safety → 拒絕
    → orexis.request_write_approval → WriteApprovalRequest 廣播
      → shittim-chest 顯示審批對話框（industrial.approveWrite）
        → [已核准] → 臨時白名單項目 → 執行 + 讀回驗證
        → [已拒絕]   → 代理收到拒絕，調整計畫
```

**已實作：**

| # | 任務 | 檔案 | 狀態 |
| --- | --- | --- | --- |
| A.2.4.1 | `orexis.request_write_approval` MCP 工具 — 建構 `WriteApprovalRequest`，廣播 `TuiMessage::IndustrialWriteApprovalPush`，暫停（oneshot + timeout）直到操作員回應 | `packages/agents/orexis/src/mcp/tools/industrial_write_tools.rs` | ✓ |
| A.2.4.2 | `industrial.approveWrite` WS 處理常式 — 透過共享的 `WriteApprovalRegistry` 解析待處理的請求；核准時新增臨時白名單項目，使後續寫入通過 `verify_write_safety` | `packages/scepter/src/tui_connection/mod.rs` | ✓ |

生產者/解析器透過跨處理程序的共享
`WriteApprovalRegistry`（`_shared_security_policy::write_approval_registry`）
解耦，在啟動時注入 orexis，並在操作員回應時由 scepter 使用。

---

## 階段 E：端對端 Dogfood

操作性驗證，非純程式碼。需要執行硬體模擬器。

### E.1 — 測試環境

| # | 元件 | 設定 |
| --- | --- | --- |
| E.1.1 | S7comm 模擬器 | 將 `snap7-server` crate 作為虛擬 S7-1500 執行。在偏移 0 處預載帶有 REAL temp、偏移 4 處 REAL pressure、偏移 8 處 INT flow、偏移 10 處 BOOL valve 的 DB1，加上 50 位元組的隨機資料 |
| E.1.2 | Modbus 模擬器 | 在虛擬序列埠（`socat pty pty`）上以 slave 模式執行 aoba。預載帶有已知寄存器值的站號 5 |
| E.1.3 | Entelecheia + evernight | 標準 docker-compose 啟動。evernight `sensor-poll` 已準備好 `--manifest` 標誌 |

### E.2 — Dogfood 情境

| # | 情境 | 步驟 | 通過標準 |
| --- | --- | --- | --- |
| E.2.1 | **未知 S7comm 走廊** | (1) 給予系統目標 `192.168.1.10:102`。(2) `industrial_discover` 技能鏈自主執行。(3) 系統發現 S7comm 協定、DB1、推斷欄位語義、生成清單。(4) 操作員在 TUI 中審查清單。(5) 核准 → evernight 開始輪詢。(6) 注入警報值 → Hubris alarm_response 觸發 → 提議糾正動作。 | 生成帶有 ≥ 3 個正確推斷欄位的清單。警報觸發 `alarm_response → task_decompose → plan_execute` 鏈。 |
| E.2.2 | **未知 Modbus 走廊** | 相同流程，但使用虛擬序列埠上的 Modbus RTU。不同的站號佈局。 | 相同標準。 |
| E.2.3 | **混合協定發現** | 同時執行兩個模擬器。系統發現兩者，生成合併的清單。 | 兩個站號都出現在清單中，協定正確。 |
| E.2.4 | **寫入審批流程** | 代理提議關閉閥門（寫入發現的 BOOL 欄位）。`verify_write_safety` 阻塞（未加入白名單）。WriteApprovalRequest 發送給操作員。操作員核准。寫入以讀回驗證執行。 | 完整往返：提議 → 阻塞 → 請求 → 核准 → 執行 → 驗證。**(D.2 現已交付 — 可進行 dogfood。)** |

### E.3 — 展示錄製

| # | 任務 | 備註 |
| --- | --- | --- |
| E.3.1 | 將完整的發現→監控→警報→回應週期錄製為螢幕截取 | 展示對未知硬體的自主適應 |
| E.3.2 | 生成發現報告產物（自動生成清單 TOML + 推斷欄位表） | 用於政府里程碑審查的具體交付物 |

---

## 對同級專案的依賴（剩餘）

| 同級專案 | 我們需要他們提供什麼 | 時間 | 狀態 |
| --- | --- | --- | --- |
| **arona** | `WriteApprovalRequest` 的 WS 廣播路徑（A.2.4） | ~~阻塞 A.2.4 / D.2~~ 完成 — 搭載 `TuiMessage::IndustrialWriteApprovalPush`（從 arona 型別重新匯出） | ✓ |
| **shittim-chest** | 操作員審批對話框（`industrial.approveWrite` 消費者）+ 發現進度渲染 | 阻塞 E.2.4 dogfood（scepter 中的 WS 處理常式已就緒；shittim-chest 需要渲染對話框並 POST 回應） | 同級 PLAN |

---

## 明確超出範圍（2 週衝刺）

- OPC UA 用戶端/伺服器（Rust 生態系統尚未就緒）
- EtherNet/IP / CIP（Rockwell）
- EtherCAT（Beckhoff）
- CAN bus
- 前端測試覆蓋率（shittim-chest 僅獲得指導性計畫，無測試編寫）
- CLI 功能與 TUI 持平

---

# 技術路線圖 — 架構深化

> **日期**：2026-06-26
> **背景**：在清理倉庫中 700+ 個過時文件/檔案並將所有提示詞合併到 `res/prompts/` 之後，我們對照實際原始碼審查了剩餘的設計文件，以確定哪些理想性設計值得實作。

---

## 1. 子徽章定址 + 平行技能執行

**判決**：值得實作。基礎設施 ~80% 已完成，缺少最後的 20%。

**目前狀態**：

- `BadgeRegistry`（`packages/scepter/src/state_machine/badge_registry.rs:92-120`）已支援父子 `link_sessions()`。
- `#001.005` 子徽章語法解析存在於 `find_by_container_id_or_sub()` 中，但剝離了子編號而非解析到獨特的子容器。
- `SnowflakeContainer.parent_id` 和 `branch_level` 欄位存在但僅作為中繼資料 — 從未用於路由。
- 邊緣節點優先級佇列（`edge_node_registry.rs:73-126`）已準備好用於細粒度的資源鎖定。
- 技能鏈嚴格為**序列式** — `pipeline.rs:68-226` 一次執行一個技能。具有獨立 `next_targets` 的協調者技能以序列方式執行，而它們可以平行執行。

**缺少的部分**：

1. ✅ 使 `find_by_container_id_or_sub()` 將 `#001.005` 解析為父容器的

最深活動分支子容器，當沒有分支存在時退回到父容器（向下相容）。

1. ✅ 新增對 `SnowflakeManager` 的子/後代查詢：`children_of`、

`children_of_badge`、`most_recent_child_of`、`deepest_descendant`
（`parent_id` → 反向索引）。

1. ✅ 基於 `FuturesUnordered` 的 `next_targets` 平行執行：

`dispatch_parallel_targets` 將協調者的獨立**葉**目標
透過 `parallel_dispatch::fan_out`（由 `Semaphore` 限制）並行展開。
序列 `invoke_skill_with_retries` 路徑中的兩個全域單例阻塞器處理如下：

   - **共享本機 cosmos 命名空間** → 每個目標在第一階段被分支到自己的

**cosmos 容器**中（`fork_container_for_skill` +
`assign_container_id` + `register_container_badge_in_registry`），因此
`dump/restore_cosmos_namespace` 對每個分支是無操作的，並行執行
是隔離的。`MAX_BRANCH_DEPTH`（第 4 項）限制分支鏈。

   - **`active_streaming_skill` UI 競爭** → 可容忍（對一個

`Option` 的最後寫入者獲勝；每個分支後重置為 `None`）。

   - **`&mut SkillChainInput` 執行緒** → `BranchOwner` 鏡像每個分支的

可變部分；`as_input` 將它們借回一個短暫的
`SkillChainInput`，以便重複使用未變更的管線輔助函式。
第一階段（分支 + 準備 + 構建提示詞 + 工具白名單）是**序列化**的
以避免 `rag_buffer` 競爭；只有第二階段（延遲主導的 LLM
調用）平行執行；第三階段清理並合併報告
（`merge_branch_reports`）到父上下文中。由
`SKILL_CHAIN_PARALLEL_TARGETS`（預設**關閉**）+
`parallel_targets_eligible`（容器化 + 全葉目標）控制。序列
`route_to_next_skill` 中的堆疊解除仍是預設行為。

1. ✅ 在兩個分支路徑中強制執行 `MAX_BRANCH_DEPTH`

（`COSMOS_MAX_BRANCH_DEPTH`，預設 4）；
子節點現在以 `source.branch_level + 1` 而非硬編碼的 `1` 註冊。

**預期影響**：來自 `industrial_discover` 等協調者技能的平行檔案寫入、平行分析將顯著減少端對端延遲。

---

## 2. 記憶體沉積管線

**判決**：品質倍增器，非關鍵。保留給長期路線圖。

**目前狀態**：

- `PhiliaMemoryService` 是一個扁平的「儲存 → 嵌入 → 檢索」圖，沒有新陳代謝。
- `memory_consolidate` 是瑣碎的 — 僅建立一個事件節點，沒有抽象/摘要化。
- 沒有記憶衰減、老化、陳舊評分或節點間的品質梯度。
- 所有節點都是無差異的 `MemoryNode` — 沒有事件性/程序性/原子性分離。
- 記憶體內向量搜尋是 O(n) 暴力法（長期無法擴展）。
- `KnowledgeStore`（獨立系統）具有生命週期階段（Created→Vectorized→Searchable→Consolidated→Deprecated）和共識驗證 — 這是現有最接近沉積的類比。

**為什麼不緊急**：

- RAG 上下文注入（`RagContextBuffer` → LLM 查詢重寫 → `bundle_search`）為目前的工具呼叫代理提供了足夠的上下文。
- pgvector HNSW 索引處理生產規模的檢索。
- 系統以「儲存和檢索」方式運作 — 沉積將使其「新陳代謝」，但這是增量品質，而非功能缺口。

**未來工作**（無時間表）：

- 自動合併：對相關節點進行定期的 LM 驅動摘要化，轉化為更高層次的「事件」。
- 品質梯度：存取計數、時間衰減、信心評分。
- 具有差異化檢索策略的三通道原型（事件性/程序性/原子性）。

---

## 3. 代理間協商

**判決**：低優先級。原始元素作為低階構建塊存在；沒有即時的使用案例。

**目前狀態**：

- `deliver_message(message_type="Question")` 存在（`epieikeia/src/mcp/tools/deliver_message.rs:63`）— 可以向另一個代理的信箱推送問題。
- `inject_user_prompt` / `consume_injected_prompts` 存在但為**輪詢式** — 沒有管線整合。代理必須顯式調用 `consume_injected_prompts` 來檢查郵件。
- `Haplotes` 具有 `AskAgent` / `ReplyAgent` / `Escalated` 對話路由型別 — 但都是無操作 ACK，沒有業務邏輯。
- `NEGOTIATION_ROUND_TIMEOUT_SECS` / `NEGOTIATION_TOTAL_TIMEOUT_SECS` 環境變數在 `RuntimeTuningConfig` 中已定義，但**從未被任何地方使用** — 死碼。

**為什麼低優先級**：

- 目前的序列技能鏈分派 + 上下文作為字串傳遞可處理所有目前的使用案例。
- 合併衝突由單一技能分派（`resolve_merge_conflict`）處理，這已足夠。
- 協商迴路（攔截技能鏈 → 詢問代理 → 等待回應 → 合併）構建和測試會很複雜。尚無生產使用案例需求。

**何時重新審視**：若代理需要在鏈中動態協商決策（而非僅分派並等待），原始元素已完成 40%。差距在於管線整合迴路。

---

## 摘要

| 功能 | 基礎設施構建 | 優先級 | 下一步 |
| --- | --- | --- | --- |
| 子徽章 + 平行執行 | 100% | **高** | ✅ 完成 — 子徽章→子節點、子索引、分支深度和迴路內平行分派全部交付（平行預設關閉） |
| 記憶體沉積 | 20% | **長期** | 無即時行動；平行執行後重新審視 |
| 代理間協商 | 40% | **低** | 等待具體使用案例；原始元素已就緒 |
