+++
title = "基础概念"
description = """> 以当前代码现实为准的概念说明"""
lang = "zhs"
category = "guides"
subcategory = "core"
+++

# 基础概念

> 以当前代码现实为准的概念说明

## 概述

Entelecheia（玄枢） 是一个多智能体平台，采用较小的模型可见工具面、共享运行时和多种客户端入口。由于仓库中同时包含当前实现、实验能力和设计文档，本指南只解释当前代码中已经活跃的核心概念。

## 核心概念

### Agent

Agent 是带有 prompt、skill 和 MCP tool 的运行时角色。

- Layer1 是当前平台核心能力。
- 当前 workspace 中活跃的内置 Layer2 是 Web Automation。
- Layer3（设计阶段）计划从 `.amphoreus/` 目录加载 — 尚未实现。

### Exec-Only 工具面

模型不会直接看到全部 MCP 工具。当前主要的模型可见工具是：

- `exec`
- `write_to_var`
- `write_to_var_json`

在运行时内部，`exec` 中的代码可以通过 ES 模块导入调用工具函数（例如 `import { tool } from 'agent'`）。

### MCP 工具

MCP 工具是内部的结构化能力接口。

- 一部分已经真实实现。
- 一部分是部分实现。
- 还有一部分仍然是 stub 或参数校验骨架。

因此，不应默认把文档中出现的每个工具都理解为已经可稳定交付。

### Skill

Skill 是基于 prompt 定义的工作流，会引用相关工具，有时也引用其他 skill。

- 有些 skill 已经可以驱动真实工作流。
- 也有一些 skill 更接近 SOP 文档，而不是完整自动化链路。

### 层级

| 层级 | 当前含义 |
| --- | --- |
| Layer1 | workspace 中编译启用的核心 Agent |
| Layer2 | Web Automation 这一活跃内置领域 Agent，加上若干归档设计 |
| Layer3 | 用户自定义 Agent（计划中，尚未实现） |

## 客户端

### TUI

当前最完整、最成熟的用户入口是 TUI。

### WebUI

Web UI（arona 聊天）与管理面板（plana）已迁移至姊妹仓库 [shittim-chest](https://github.com/celestia-island/shittim-chest) 并从本代码库移除；本仓库的首选界面是 TUI。

### CLI

CLI 已存在，但部分命令仍是占位输出。

### Tauri 客户端

桌面端和移动端代码已存在于 [shittim-chest](https://github.com/celestia-island/shittim-chest) 兄弟仓库中，但更适合视为早期集成。IDE 集成（VS Code、IntelliJ）同样位于 shittim-chest。

## 安全模型的保守表述

- 已有 JWT 与 API key 认证能力。
- 已有已知 HTTP、WebSocket、MCP 路径的 RBAC 映射。
- 已有加密的 provider key 存储能力。
- 容器硬化与审计完整性仍不完整。

除非已核对具体代码路径，否则不要把双向 TLS、完整 capability token 或全链路严格策略执行视为当前事实。
