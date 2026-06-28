+++
title = "架构决策记录（ADR）"
description = """本目录记录了 Entelecheia 开发过程中做出的关键架构决策。每个 ADR 解释了决定了什么、为什么决定以及考虑了哪些权衡。"""
lang = "zhs"
category = "design"
subcategory = "core"
+++

# 架构决策记录（ADR）

本目录记录了 Entelecheia 开发过程中做出的关键架构决策。每个 ADR 解释了**决定了什么**、**为什么**决定以及考虑了哪些**权衡**。

ADR 遵循 [Michael Nygard ADR 模板](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)。一旦发布即不可变——被取代的决策会相应标记。

## 索引

| ADR | 标题 | 状态 |
| --- | --- | --- |
| [ADR-001](en/exec-only-microkernel-tool-surface.md) | 仅执行微内核工具表面 | 已接受 |
| [ADR-002](en/boa-javascript-engine.md) | Boa 作为嵌入式 JavaScript 引擎 | 已接受 |
| [ADR-003](en/postgresql-pgvector-storage.md) | PostgreSQL + PgVector 用于统一数据存储 | 已接受 |
| [ADR-004](en/layered-crate-workspace.md) | 60+ Crate 分层工作区架构 | 已接受 |
| [ADR-005](en/container-sandboxed-agent-execution.md) | 使用 COSMOS 实现容器沙箱 Agent 执行 | 已接受 |

## 语言目录

| 代码 | 语言 |
| --- | --- |
| `en/` | 英语（权威） |
| `zhs/` | 简体中文 |
| `zht/` | 繁体中文 |
| `ja/` | 日语 |
| `ko/` | 韩语 |
| `fr/` | 法语 |
| `es/` | 西班牙语 |
| `ru/` | 俄语 |
