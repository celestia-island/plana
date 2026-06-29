+++
title = "TerminalBackend / ViewportBackend / FileBackend Trait 抽象"
description = """架構決策記錄 —— TerminalBackend / ViewportBackend / FileBackend Trait 抽象。"""
lang = "zht"
category = "design"
subcategory = "router"
+++

# TerminalBackend / ViewportBackend / FileBackend Trait 抽象

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

Evernight 需要多型的後端介面，以便前端（CLI、TUI、GUI）能夠統一地使用任何協定。如果沒有共享的 trait，每個前端都需要為終端機、圖形顯示和檔案操作編寫協定特定的程式碼路徑。

## 決策

在 crate 根目錄中定義三個物件安全的非同步 trait（始終可用，無需功能標誌）：

- **`TerminalBackend`** — `read` / `write` / `resize` / `close`
- **`ViewportBackend`** — `render` / `input` / `clipboard` / `close`
- **`FileBackend`** — `list` / `stat` / `get` / `put` / `rm` / `mkdir` / `rename`

每個協定後端實作相關的 trait。前端使用 `Box<dyn TerminalBackend>` 等形式。

## 後果

### 正面

- 前端與協定無關；新的後端（例如 RDP）可以加入而無需修改前端。

### 負面

- 非同步 trait 物件需要 `Box::pin` 的額外負擔。
- trait 設計必須穩定，因為變更它會破壞所有實作者。
