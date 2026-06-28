+++
title = "架構決策記錄（ADR）"
description = """本目錄記錄了 Entelecheia 開發過程中所做的關鍵架構決策。每個 ADR 說明了做出了什麼決策、為什麼做出該決策，以及考慮了哪些取捨。"""
lang = "zht"
category = "design"
subcategory = "core"
+++

# 架構決策記錄（ADR）

本目錄記錄了 Entelecheia 開發過程中所做的關鍵架構決策。每個 ADR 說明了**做出了什麼**決策、**為什麼**做出該決策，以及考慮了哪些**取捨**。

ADR 遵循 [Michael Nygard ADR 範本](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)。一經發布即不可變更 — 被取代的決策會相應標記。

## 索引

| ADR | 標題 | 狀態 |
| --- | --- | --- |
| [ADR-001](en/exec-only-microkernel-tool-surface.md) | 僅執行微核心工具表面 | 已接受 |
| [ADR-002](en/boa-javascript-engine.md) | Boa 作為嵌入式 JavaScript 引擎 | 已接受 |
| [ADR-003](en/postgresql-pgvector-storage.md) | PostgreSQL + PgVector 用於統一資料儲存 | 已接受 |
| [ADR-004](en/layered-crate-workspace.md) | 60+ Crate 分層工作區架構 | 已接受 |
| [ADR-005](en/container-sandboxed-agent-execution.md) | 使用 COSMOS 的容器沙箱化代理執行 | 已接受 |

## 語言目錄

| 代碼 | 語言 |
| --- | --- |
| `en/` | 英文（權威版本） |
| `zhs/` | 簡體中文 |
| `zht/` | 繁體中文 |
| `ja/` | 日文 |
| `ko/` | 韓文 |
| `fr/` | 法文 |
| `es/` | 西班牙文 |
| `ru/` | 俄文 |
