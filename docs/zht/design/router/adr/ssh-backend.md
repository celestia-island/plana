+++
title = "SSH 後端 — russh 搭配共享連線"
description = """架構決策記錄 —— SSH 後端 — russh 搭配共享連線。"""
lang = "zht"
category = "design"
subcategory = "router"
+++

# SSH 後端 — russh 搭配共享連線

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

Evernight 需要 SSH 用於遠端 shell、檔案操作和終端機存取。最初，程式碼庫有三個獨立的 SSH 處理器實作（FileHandler、SshHandler、TerminalHandler），包含重複的連線邏輯。該專案要求純 Rust（不使用 `Command::new("ssh")` 的 shell 外呼）。

## 決策

使用 `russh`（純 Rust SSH-2 實作）作為 SSH 後端。將所有 SSH 處理器實作合併為 `remote/connection.rs` 中的單一 `DefaultSshHandler`，並使用共享的 `connect_session()` 函式。所有 SSH 操作（shell、檔案、終端機）共享此連線抽象。

## 後果

### 正面

- SSH 認證邏輯的單一事實來源
- 稍後易於新增連線池

### 負面

- `russh` 在邊緣情況下可能落後於 OpenSSH
- 尚無內建的 SSH agent 轉發
