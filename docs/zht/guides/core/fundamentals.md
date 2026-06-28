+++
title = "基礎概念"
description = """> 以當前程式碼現實為準的概念說明"""
lang = "zht"
category = "guides"
subcategory = "core"
+++

# 基礎概念

> 以當前程式碼現實為準的概念說明

## 概述

Entelecheia（玄樞） 是一個多智慧體平台，採用較小的模型可見工具面、共享執行時和多種客戶端入口。由於倉庫中同時包含當前實作、實驗能力和設計文件，本指南只解釋當前程式碼中已經活躍的核心概念。

## 核心概念

### Agent

Agent 是帶有 prompt、skill 和 MCP tool 的執行時角色。

- Layer1 是當前平台核心能力。
- 當前 workspace 中活躍的內建 Layer2 是 Web Automation。
- Layer3（設計階段）計劃從 `.amphoreus/` 目錄載入 — 尚未實作。

### Exec-Only 工具面

模型不會直接看到全部 MCP 工具。當前主要的模型可見工具是：

- `exec`
- `write_to_var`
- `write_to_var_json`

在執行時內部，`exec` 中的程式碼可以透過 ES 模組匯入呼叫工具函式（例如 `import { tool } from 'agent'`）。

### MCP 工具

MCP 工具是內部的結構化能力介面。

- 一部分已經真實實作。
- 一部分是部分實作。
- 還有一部分仍然是 stub 或參數校驗骨架。

因此，不應預設把文件中出現的每個工具都理解為已經可穩定交付。

### Skill

Skill 是基於 prompt 定義的工作流，會引用相關工具，有時也引用其他 skill。

- 有些 skill 已經可以驅動真實工作流。
- 也有一些 skill 更接近 SOP 文件，而不是完整自動化鏈路。

### 層級

| 層級 | 當前含義 |
| --- | --- |
| Layer1 | workspace 中編譯啟用的核心 Agent |
| Layer2 | Web Automation 這一活躍內建領域 Agent，加上若干歸檔設計 |
| Layer3 | 使用者自訂 Agent（計劃中，尚未實作） |

## 客戶端

### TUI

當前最完整、最成熟的使用者入口是 TUI。

### WebUI

Web UI（arona 聊天）與管理面板（plana）已遷移至姊妹倉庫 [shittim-chest](https://github.com/celestia-island/shittim-chest) 並從本程式碼庫移除；本倉庫的首選介面是 TUI。

### CLI

CLI 已存在，但部分命令仍是佔位輸出。

### Tauri 客戶端

桌面端和行動端程式碼已存在於 [shittim-chest](https://github.com/celestia-island/shittim-chest) 兄弟倉庫中，但更適合視為早期整合。IDE 整合（VS Code、IntelliJ）同樣位於 shittim-chest。

## 安全模型的保守表述

- 已有 JWT 與 API key 認證能力。
- 已有已知 HTTP、WebSocket、MCP 路徑的 RBAC 映射。
- 已有加密的 provider key 儲存能力。
- 容器強化與稽核完整性仍不完整。

除非已核對具體程式碼路徑，否則不要把雙向 TLS、完整 capability token 或全鏈路嚴格策略執行視為當前事實。
