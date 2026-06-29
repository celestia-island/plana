+++
title = "SSH 連線池"
description = """架構決策記錄 —— SSH 連線池。"""
lang = "zht"
category = "design"
subcategory = "router"
+++

# SSH 連線池

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

原始程式碼為每個操作（列出檔案、執行指令）開啟一個新的 SSH 連線。這成本高昂（每次呼叫都要 TCP + 金鑰交換 + 認證），且對於具有多個並行操作的互動式使用無法擴展。

## 決策

實作以 `(host, port, username)` 為索引鍵的 `SshConnectionPool`。連線在首次請求時延遲建立，並在操作之間重用。`PooledSshClient` 包裝一個共享的 `Arc<Mutex<SshSession>>`。定期的健康檢查會移除失效的連線。

## 後果

### 正面

- 重複操作的延遲大幅降低。
- 允許在一個連線上的並行 shell + 檔案 + 終端機會話。

### 負面

- 池必須處理連線過期和重新連線。
- 長期連線可能被防火牆中斷。
