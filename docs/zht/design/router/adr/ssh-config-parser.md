+++
title = "SSH 設定解析器"
description = """架構決策記錄 —— SSH 設定解析器。"""
lang = "zht"
category = "design"
subcategory = "router"
+++

# SSH 設定解析器

- **狀態**：已接受
- **日期**：2026-06-09
- **作者**：Evernight Core Team

## 背景

使用者期望 `evernight` 能夠像 `ssh` 指令一樣讀取 `~/.ssh/config`，以取得主機別名、跳板主機和金鑰檔案。如果沒有這項功能，使用者必須在每次呼叫時重複輸入所有連線參數。

## 決策

實作一個純 Rust 的 SSH 設定解析器，處理標準指令：`Host`、`HostName`、`User`、`Port`、`IdentityFile`、`ProxyJump`、`ForwardAgent`、`ServerAliveInterval` 等。對 `Host` 條目使用 glob 模式匹配。與連線池整合，以實現基於設定的透明連線。

## 後果

### 正面

- 即裝即用的 SSH 相容性；從設定中解析跳板主機。

### 負面

- 必須追蹤 OpenSSH 設定語法的變更。
- `Match` 指令的支援較為複雜。
