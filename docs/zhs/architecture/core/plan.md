+++
title = "工业走廊集成计划"
description = """> 目标：系统必须展示对完全未知的工业示范走廊的自主自适应——
lang = "zhs"
category = "architecture"
subcategory = "core"
+++

# 工业走廊自主集成计划

> **目标**：系统必须展示对完全未知的工业示范走廊的**自主自适应接口能力** — 发现硬件、
> 推断数据模型、生成监控配置，并闭合告警→响应循环 — 无需
> 人工逐设备工程。
> **政府硬性截止日期**：此能力与政府项目里程碑绑定。

---

## 剩余工作

完整的发现 → 推断 → 监控 → 告警 → **写入审批**链已
交付（阶段 A.1–A.3、B、C、D.1、**D.2 ✓**）。仅剩的工作是
**端到端自举验证（阶段 E）** — 运营性质，非代码性质。

### D.2 — 写入审批往返（人机协同）✓

```text
智能体决定需要写入
  → verify_write_safety → 拒绝
    → orexis.request_write_approval → WriteApprovalRequest 广播
      → shittim-chest 显示审批对话框（industrial.approveWrite）
        → [批准] → 临时白名单条目 → 执行 + 回读验证
        → [拒绝]   → 智能体收到拒绝，调整计划
```

**已实现：**

| # | 任务 | 文件 | 状态 |
| --- | --- | --- | --- |
| A.2.4.1 | `orexis.request_write_approval` MCP 工具 — 构建 `WriteApprovalRequest`，广播 `TuiMessage::IndustrialWriteApprovalPush`，挂起（oneshot + timeout）直到操作员响应 | `packages/agents/orexis/src/mcp/tools/industrial_write_tools.rs` | ✓ |
| A.2.4.2 | `industrial.approveWrite` WS 处理程序 — 通过共享的 `WriteApprovalRegistry` 解析挂起的请求；批准时添加临时白名单条目，以便后续写入通过 `verify_write_safety` | `packages/scepter/src/tui_connection/mod.rs` | ✓ |

生产者/解析器通过进程范围的共享
`WriteApprovalRegistry`（`_shared_security_policy::write_approval_registry`）
解耦，在启动时注入到 orexis，操作员响应时由 scepter 使用。

---

## 阶段 E：端到端自举

运营验证，非纯代码。需要运行硬件模拟器。

### E.1 — 测试环境

| # | 组件 | 设置 |
| --- | --- | --- |
| E.1.1 | S7comm 模拟器 | 将 `snap7-server` crate 作为虚拟 S7-1500 运行。预加载 DB1：偏移 0 处 REAL 温度，偏移 4 处 REAL 压力，偏移 8 处 INT 流量，偏移 10 处 BOOL 阀门，外加 50 字节随机数据 |
| E.1.2 | Modbus 模拟器 | 在虚拟串口（`socat pty pty`）上以从站模式运行 aoba。预加载站 5 已知寄存器值 |
| E.1.3 | Entelecheia + evernight | 标准 docker-compose 启动。evernight `sensor-poll` 就绪，带 `--manifest` 标志 |

### E.2 — 自举场景

| # | 场景 | 步骤 | 通过标准 |
| --- | --- | --- | --- |
| E.2.1 | **未知 S7comm 走廊** | （1）给系统目标 `192.168.1.10:102`。（2）`industrial_discover` 技能链自主运行。（3）系统发现 S7comm 协议、DB1，推断字段语义，生成清单。（4）操作员在 TUI 中审查清单。（5）批准 → evernight 开始轮询。（6）注入告警值 → Hubris alarm_response 触发 → 提出纠正措施。 | 生成的清单包含 ≥ 3 个正确推断的字段。告警触发 `alarm_response → task_decompose → plan_execute` 链。 |
| E.2.2 | **未知 Modbus 走廊** | 相同流程，但在虚拟串口上使用 Modbus RTU。不同的站布局。 | 相同标准。 |
| E.2.3 | **混合协议发现** | 同时运行两个模拟器。系统发现两者，生成组合清单。 | 两个站均出现在清单中，协议正确。 |
| E.2.4 | **写入审批流程** | 智能体提议关闭阀门（写入已发现的 BOOL 字段）。`verify_write_safety` 阻止（未列入白名单）。WriteApprovalRequest 发送给操作员。操作员批准。写入以回读验证执行。 | 完整往返：提议 → 阻止 → 请求 → 批准 → 执行 → 验证。**（D.2 现已交付 — 可自举。）** |

### E.3 — 演示录制

| # | 任务 | 备注 |
| --- | --- | --- |
| E.3.1 | 将完整发现→监控→告警→响应循环录制为屏幕录像 | 展示对未知硬件的自主自适应 |
| E.3.2 | 生成发现报告制品（自动生成的清单 TOML + 推断字段表） | 政府里程碑评审的有形交付物 |

---

## 对兄弟项目的依赖（剩余）

| 兄弟 | 我们需要从他们那里获得什么 | 何时 | 状态 |
| --- | --- | --- | --- |
| **arona** | `WriteApprovalRequest` 的 WS 广播路径（A.2.4） | ~~阻塞 A.2.4 / D.2~~ done — 通过 `TuiMessage::IndustrialWriteApprovalPush`（从 arona 类型重新导出） | ✓ |
| **shittim-chest** | 操作员审批对话框（`industrial.approveWrite` 消费者）+ 发现进度渲染 | 阻塞 E.2.4 自举（scepter 中的 WS 处理程序已就绪；shittim-chest 需要渲染对话框并 POST 响应） | 兄弟 PLAN |

---

## 明确不在此范围内（2 周冲刺）

- OPC UA 客户端/服务器（Rust 生态不成熟）
- EtherNet/IP / CIP（Rockwell）
- EtherCAT（Beckhoff）
- CAN 总线
- 前端测试覆盖（shittim-chest 仅获得指导计划，不编写测试）
- CLI 功能对等 TUI

---

# 技术路线图 — 架构深化

> **日期**：2026-06-26
> **背景**：在清理仓库中 700+ 份陈旧文档/文件并将所有提示整合到 `res/prompts/` 后，我们对照实际源代码审计了剩余的设计文档，以识别哪些期望设计值得实现。

---

## 1. 子徽章寻址 + 并行技能执行

**结论**：值得实现。基础设施约 80% 已构建，仅缺失最后 20%。

**当前状态**：

- `BadgeRegistry`（`packages/scepter/src/state_machine/badge_registry.rs:92-120`）已支持父子 `link_sessions()`。
- `#001.005` 子徽章语法解析存在于 `find_by_container_id_or_sub()` 中，但会剥离子编号，而不是解析为不同的子容器。
- `SnowflakeContainer.parent_id` 和 `branch_level` 字段存在但仅作为元数据 — 从未用于路由。
- 边缘节点优先级队列（`edge_node_registry.rs:73-126`）已为细粒度资源锁定准备就绪。
- 技能链严格**串行** — `pipeline.rs:68-226` 一次循环一个技能。协调器技能具有独立 `next_targets` 但串行运行，而这些本可以并行运行。

**缺失内容**：

1. ✅ 使 `find_by_container_id_or_sub()` 解析 `#001.005` → 父容器的最深层活跃分支子代，当不存在分支时回退到父代（向后兼容）。
1. ✅ 向 `SnowflakeManager` 添加子代/后代查找：`children_of`、`children_of_badge`、`most_recent_child_of`、`deepest_descendant`（`parent_id` → 反向索引）。
1. ✅ 基于 `FuturesUnordered` 的 `next_targets` 并行执行：`dispatch_parallel_targets` 通过 `parallel_dispatch::fan_out`（由 `Semaphore` 限界）将协调器的独立**叶子**目标并发扇出。串行 `invoke_skill_with_retries` 路径中的两个全局单例阻塞，处理方式如下：

   - **共享本地 cosmos 命名空间** → 每个目标在第一阶段分支出其**自己的 cosmos 容器**（`fork_container_for_skill` + `assign_container_id` + `register_container_badge_in_registry`），因此 `dump/restore_cosmos_namespace` 在每个分支中是空操作，并发执行是隔离的。`MAX_BRANCH_DEPTH`（第 4 项）限制了分支链。
   - **`active_streaming_skill` UI 竞态** → 容忍（`Option` 上最后写入者胜；每个分支后重置为 `None`）。
   - **`&mut SkillChainInput` 线程安全** → `BranchOwner` 按分支镜像可变部分；`as_input` 将它们借回短生命周期的 `SkillChainInput`，以便复用来改变的管道辅助函数。

阶段 1（分支 + 准备 + 构建提示 + 工具白名单）被**序列化**以避免 `rag_buffer` 竞态；仅阶段 2（延迟主导的 LLM 调用）并行运行；阶段 3 清理并通过 `merge_branch_reports` 将报告合并到父上下文中。由 `SKILL_CHAIN_PARALLEL_TARGETS`（默认**关闭**）+ `parallel_targets_eligible`（容器化 + 全叶子目标）门控。`route_to_next_skill` 中的串行栈展开仍然是默认行为。

1. ✅ 在两个分支路径中强制执行 `MAX_BRANCH_DEPTH`（`COSMOS_MAX_BRANCH_DEPTH`，默认 4）；子代现在以 `source.branch_level + 1` 注册，而非硬编码的 `1`。

**预期影响**：来自协调器技能（如 `industrial_discover`）的并行文件写入、并行分析将显著降低端到端延迟。

---

## 2. 记忆沉淀管道

**结论**：质量倍增器，非关键。保留在长期路线图中。

**当前状态**：

- `PhiliaMemoryService` 是一个扁平的"存储 → 嵌入 → 检索"图，无新陈代谢。
- `memory_consolidate` 是平凡的 — 仅创建一个片段节点，无抽象/摘要。
- 没有记忆衰减、老化、陈旧度评分或节点间的质量梯度。
- 所有节点是无区别的 `MemoryNode` — 无片段性/程序性/原子性分离。
- 内存向量搜索是 O(n) 暴力搜索（长期不可扩展）。
- `KnowledgeStore`（独立系统）具有生命周期阶段（Created→Vectorized→Searchable→Consolidated→Deprecated）和共识验证 — 这是与沉淀最接近的现有类比。

**为何不紧急**：

- RAG 上下文注入（`RagContextBuffer` → LLM 查询改写 → `bundle_search`）为当前的工具调用智能体提供足够的上下文。
- pgvector HNSW 索引处理生产级检索。
- 系统以"存储和检索"方式工作 — 沉淀会使其"代谢"，但这是增量质量提升，而非功能差距。

**未来工作**（无时间表）：

- 自动整合：定期由 LM 驱动的相关节点摘要化为更高级别的"片段"。
- 质量梯度：访问计数、时间衰减、置信度评分。
- 三通道原型（片段性/程序性/原子性），具有差异化的检索策略。

---

## 3. 智能体间协商

**结论**：低优先级。原语作为低级构建块存在；无即时用例。

**当前状态**：

- `deliver_message(message_type="Question")` 存在（`epieikeia/src/mcp/tools/deliver_message.rs:63`）— 可以向另一个智能体的邮箱推送问题。
- `inject_user_prompt` / `consume_injected_prompts` 存在，但是**基于轮询** — 无管道集成。智能体必须显式调用 `consume_injected_prompts` 来检查邮件。
- `Haplotes` 具有 `AskAgent` / `ReplyAgent` / `Escalated` 对话路由类型 — 但全部是零业务逻辑的空操作确认。
- `NEGOTIATION_ROUND_TIMEOUT_SECS` / `NEGOTIATION_TOTAL_TIMEOUT_SECS` 环境变量在 `RuntimeTuningConfig` 中定义，但**从未在任何地方被消费** — 死代码。

**为何低优先级**：

- 当前的串行技能链分发 + 上下文作为字符串传递处理所有当前用例。
- 合并冲突由单技能分发（`resolve_merge_conflict`）处理，这已足够。
- 协商循环（拦截技能链 → 询问智能体 → 等待响应 → 合并）构建和测试会很复杂。尚无生产用例要求它。

**何时重新审视**：如果智能体需要在中途动态协商链决策（不只是分发并等待），原语已完成 40%。差距在于管道集成循环。

---

## 摘要

| 功能 | 基础设施已构建 | 优先级 | 下一步 |
| --- | --- | --- | --- |
| 子徽章 + 并行执行 | 100% | **高** | ✅ 完成 — 子徽章→子代、子代索引、分支深度和循环内并行分发全部交付（并行默认关闭） |
| 记忆沉淀 | 20% | **长期** | 无即时操作；并行执行后重新审视 |
| 智能体间协商 | 40% | **低** | 等待具体用例；原语已就绪 |
