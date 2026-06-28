+++
title = "ADR-004：60+ Crate 分层工作区架构"
description = """日期：2026-03"""
lang = "zhs"
category = "design"
subcategory = "core"
+++

# ADR-004：60+ Crate 分层工作区架构

**日期**：2026-03
**状态**：已接受

## 背景

Entelecheia 最初采用单体 `packages/shared` crate（38K 行，187 个 `.rs` 文件），包含所有共享基础设施：类型、MCP 协议、LLM 提供商、容器管理、数据库、安全、配置等。随着项目增长到 12 个 Agent + 1 个领域 Agent + 3 个二进制包，出现了若干问题：

1. **编译时间**：对 `shared` 的任何修改都需要重新编译所有 187 个文件，即使只修改了一个结构体。
1. **依赖污染**：只需要 MCP 类型的 Agent crate 被迫传递依赖数据库驱动、容器运行时和 LLM 提供商。
1. **所有权不明确**：单个 crate 中的 187 个文件使得哪个模块"拥有"哪个功能不清晰，重构风险高。
1. **feature flag 爆炸**：使用 Cargo features 进行条件编译以避免引入不必要的依赖，但这导致测试配置的组合爆炸。

## 决策

将单体 `packages/shared` 分解为 **37 个专注的子 crate**，组织在 **6 个依赖层级**（L0 到 L5）中，遵循严格的依赖方向：

```text
L0（叶子） → L1 → L2 → L3 → L4 → L5 → 消费者（scepter, agents, tui）
```

**层级定义：**

| 层级 | Crate | 规则 |
| --- | --- | --- |
| **L0** | core, logging, macros | 不依赖任何其他 entelecheia crate |
| **L1** | domain_enums, mcp_types, text, concurrent | 仅依赖 L0 |
| **L2** | config, agent_registry, state_types | 依赖 L0-L1 |
| **L3** | domain_agent, container, agent_lifecycle, agent_runtime, thread_types, toolchain, infra_utils | 依赖 L0-L2 |
| **L4** | state_sync, domain_skills, hooks, domain_auth, container_runtime, skills_permissions, timeline, iepl | 依赖 L0-L3 |
| **L5** | llm_provider, prompt, custom_agent, storage, infra_jsonrpc, infra_services, e2e_events, adapter, plugin_host, rag, embedding, security_policy | 依赖 L0-L4 |

所有内部依赖声明使用 `workspace = true` 以保证版本一致性。不存在薄聚合 crate——消费者直接从各个子 crate 导入。

## 后果

### 积极方面

- **增量编译**：对 `shared-core`（L0）的修改仍会传播，但对 `shared-security-policy`（L5）的修改仅重新编译该 crate 及其直接消费者。构建时间显著改善。
- **明确的所有权边界**：每个 crate 有明确的职责。代码审查范围自然地受 crate 边界约束。
- **依赖隔离**：Agent crate 仅导入所需的共享 crate。SkeMma 不会拉入数据库驱动。EleOs 不会拉入容器运行时。
- **循环依赖预防**：分层架构使得创建循环依赖在结构上不可能——L3 crate 不能依赖 L5 crate。
- **可独立测试**：每个 crate 的测试独立运行，无需完整工作区的依赖树。

### 消极方面

- **工作区管理开销**：单个工作区中有 60+ 个 crate 意味着需要维护更多 `Cargo.toml` 文件、版本升级时更新更多 `[dependencies]` 部分，以及更谨慎的依赖声明。
- **跨 crate 重构更难**：将一个类型从 L2 移到 L3 需要更新所有 L2 消费者，并验证没有 L3+ crate 通过旧位置意外依赖该类型。
- **Crate 命名冗长**：内部 crate 名称使用 `_shared_*` 前缀约定（如 `_shared_domain_skills_permissions`），这对于工作区清晰性来说虽然冗长但是必要的。
- **潜在的过度分解**：某些 crate（如约 200 行的 `shared-text`）可能不足以证明其自身 crate 开销的合理性。分解遵循"如果可能增长就分离"的理念，而非严格必要性。

### 接受的权衡

**以管理复杂度换取编译时间和架构清晰性。** 将 `shared` 分解为 37 个 crate 是 Rust 工作区设计中较为激进的方案。中间方案（10-15 个 crate）会更简单管理。然而，鉴于项目广阔的表面积（26 个 LLM 提供商、2 个容器运行时、12 个 Agent、完整的安全流水线、数据库、IEPL），细粒度分解确保每个部分可以独立演进。`workspace = true` 模式缓解了版本管理开销。
