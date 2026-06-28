+++
title = "ADR-005：使用 COSMOS 实现容器沙箱 Agent 执行"
description = """日期：2026-02"""
lang = "zhs"
category = "design"
subcategory = "core"
+++

# ADR-005：使用 COSMOS 实现容器沙箱 Agent 执行

**日期**：2026-02
**状态**：已接受

## 背景

在多 agent 系统中，agent 执行 LLM 生成的代码，agent 之间的隔离对于以下方面至关重要：

1. **安全性**：不可信的 LLM 输出不应能够访问另一个 agent 的内存、文件或网络连接。
1. **状态隔离**：每个 agent 的 REPL 状态（JavaScript 变量、绑定、快照）必须独立。
1. **资源控制**：行为异常的 agent 不应消耗无限的 CPU、内存或 PID。
1. **可复现性**：Agent 状态应可快照化和恢复，以供调试和回滚。
1. **Fork/Merge 工作流**：系统需要支持分支 agent 执行（fork）和合并结果（merge），类似于 git 分支。

评估了多种隔离方法：

| 方法 | 隔离强度 | 资源控制 | 快照/Fork | 开销 |
| --- | --- | --- | --- | --- |
| **每 agent 容器（Docker/OCI）** | 强（内核级） | 完全（cgroups、seccomp、capabilities） | 原生（commit/snapshot） | 中等（~100ms 启动，~50MB 每容器） |
| **每 agent 进程** | 中等（UID/seccomp） | 部分（rlimit） | 手动（序列化状态） | 低 |
| **每 agent 线程** | 弱（共享内存） | 最小 | 手动 | 最小 |
| **每 agent WASM 沙箱** | 强（线性内存） | 好（gas 计量） | 手动 | 低 |
| **每 agent Boa 上下文** | 中等（JS 沙箱） | 有限 | 内置（命名空间序列化） | 最小 |

## 决策

我们选择了**双层容器架构**，以 **COSMOS** 作为每个 agent 容器内的 init 进程：

**外层（编排基础设施）：**

- 通过 Bollard 使用 Docker/Podman 管理基础设施容器（PostgreSQL、Scepter 守护进程）。
- 完整编排能力：网络、卷、健康检查、Compose。

**内层（agent 沙箱）：**

- Youki/libcontainer（默认）或 Docker 用于每 agent COSMOS 容器。
- 每个 agent 获得自己的容器，COSMOS 作为 PID 1。
- COSMOS 是**中介所有交互的前端进程**——它提供 JSON-RPC Unix 套接字服务器、Boa JS REPL、MCP 路由器和 HapLotes 桥接连接回到 Scepter。

**为什么 COSMOS 作为强制性中介：**

与容器化 agent 的所有交互必须通过 COSMOS。直接容器操作（例如 `docker exec` 进入容器）会绕过安全模型、状态管理和审计追踪。COSMOS 提供：

1. **工具调度中介**：`McpRouter` 在任何工具到达 agent 之前强制执行允许列表、双重授权和信任级别。
1. **状态持久化**：双缓冲快照系统确保 REPL 状态在崩溃后存活。
1. **桥接通信**：HapLotes 桥接将 COSMOS 连接回 Scepter 以进行 agent 间协调。
1. **安全强制执行**：Seccomp 配置文件、出口策略和能力限制在容器创建时应用并由内核强制执行。

**为什么内层沙箱使用 Youki/libcontainer：**

- 无根且无守护进程——agent 沙箱不需要 Docker 守护进程。
- OCI 兼容——标准 `config.json` 规范，与 OCI 工具兼容。
- 快速基于 overlay 的 rootfs——快照和 fork 操作仅复制更改的文件。
- 比 Docker 对短暂容器的开销更低。

## 后果

### 正面

- **通过内核强制执行实现强隔离**：cgroups（CPU/内存/PID 限制）、seccomp（系统调用过滤）、capabilities（`cap_drop`=ALL）、命名空间（PID/网络/挂载隔离）。
- **原生 fork/merge**：容器 commit 创建镜像快照；可以从快照创建新容器。Overlay 文件系统仅跟踪更改的文件。
- **每 agent 资源限制**：默认 512MB 内存、1 CPU、100 PID，可按容器配置。
- **审计追踪**：所有工具调用通过 COSMOS 的 MCP 路由器，记录每次调度以供 OreXis 安全审计。
- **崩溃隔离**：Boa panic 或 agent 错误被限制在其容器内。其他 agent 和 Scepter 继续运行。
- **Youki 用于轻量级沙箱**：内部容器比完整 Docker 容器启动更快、消耗更少资源。

### 负面

- **COSMOS 作为 PID 1 的复杂性**：COSMOS 必须处理信号转发、僵尸收割和作为容器 init 进程的干净关闭。这增加了普通应用程序没有的责任。
- **容器启动延迟**：每个 agent 容器需要约 100ms-1s 启动（取决于运行时）。这比基于进程或基于线程的隔离慢。
- **资源开销**：每个 COSMOS 容器消耗约 50-100MB 内存用于 Boa 运行时、JS 堆和 OS 开销。有 9 个容器化 agent 时，增加约 0.5-1GB 基线内存。
- **测试复杂性**：测试 agent 行为需要运行带有 COSMOS 的实际容器，这意味着测试需要 Docker 或 Youki 可用。雪花测试模式（构建 entelecheia 镜像、运行 COSMOS 容器、通过 Unix 套接字连接）比单元测试更复杂。
- **需要维护两个运行时**：Docker/Bollard 和 Youki/libcontainer 代码路径都必须维护和测试。

### 已接受的权衡

**为安全性和隔离保证而接受资源开销。** 每 agent 进程模型将使用更少的内存并启动更快，但不会提供 agent 之间的内核级隔离。在 agent 执行不可信 LLM 生成代码的系统中，容器隔离的安全保证值得资源成本。COSMOS 作为强制性中介的设计确保即使攻击者在容器内获得代码执行，也无法绕过安全模型在 COSMOS 中介之外操作。
