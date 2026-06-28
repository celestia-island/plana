+++
title = "Cosmos 容器调度与 Token 路由设计"
description = """本文档描述了 Cosmos 容器调度架构：标记为 `ToolLocation::Cosmos` 的 MCP 工具如何通过 unix-socket JSON-RPC 路由到其对应容器，以及 token（agent 编号）系统如何与容器身份和路由关联。"""
lang = "zhs"
category = "design"
subcategory = "core"
+++

# Cosmos 容器调度与 Token 路由设计

## 概述

本文档描述了 Cosmos 容器调度架构：标记为 `ToolLocation::Cosmos` 的 MCP 工具如何通过 unix-socket JSON-RPC 路由到其对应容器，以及 token（agent 编号）系统如何与容器身份和路由关联。

## I. 工具位置模型

### 双执行环境

```mermaid
flowchart LR
    subgraph Scepter["Scepter（中央进程）"]
        A1[LLM 调用]
        A2[RAG 查询]
        A3[任务管理]
        A4[凭据存储]
    end

    subgraph Cosmos["Cosmos（每 Agent 容器）"]
        P1[文件系统访问]
        P2[脚本执行]
        P3[硬件访问]
        P4[REPL 会话]
    end

    Scepter -->|ToolLocation::Scepter| Local[本地调用]
    Cosmos -->|ToolLocation::Cosmos| Socket[Unix 套接字 RPC]
```

### ToolLocation 枚举

| 变体 | 执行位置 | 传输方式 |
| --- | --- | --- |
| `Scepter`（默认） | 进程内通过 `McpToolInvoker` | 直接函数调用 |
| `Cosmos` | 容器内通过 `CosmosConnector` | Unix 套接字 JSON-RPC |

### 位置决策标准

```mermaid
flowchart TD
    Tool[MCP 工具] --> Q1{需要容器资源？}
    Q1 -->|是：文件系统、脚本、硬件| Cosmos[ToolLocation::Cosmos]
    Q1 -->|否：LLM、RAG、状态管理| Scepter[ToolLocation::Scepter]
```

需要容器资源（文件系统、脚本执行、硬件访问）的工具标记为 `Cosmos`。集中式服务（LLM、RAG、任务管理、人类交互）保持为 `Scepter`。

## II. Token 系统与容器身份

### Agent 编号分配

```mermaid
sequenceDiagram
    participant SM as SkillChain 管理器
    participant AIM as AgentIdManager
    participant SC as SnowflakeContainer
    participant PC as CosmosConnector

    SM->>AIM: 请求 agent 编号
    AIM-->>SM: 分配 000-999 token
    SM->>SC: 为 token 创建容器
    SC-->>SM: 容器 UUID + 套接字路径
    SM->>PC: connect(UUID, socket_path)
    PC-->>SM: 连接建立
```

### Token 属性

| 属性 | 描述 |
| --- | --- |
| 格式 | 三位数字：`000`-`999` |
| 分配器 | 技能链中的 `AgentIdManager` |
| 绑定 | 每个技能链面板一个 token |
| 显示 | 在 TUI 统计行中显示为 `cosmos#NNN` |
| 持久性 | 在 agent 重启后持续存在 |

## III. 请求路由流程

### TUI 发起的 MCP 调用

```mermaid
sequenceDiagram
    participant TUI as TUI 客户端
    participant MSR as mcp_skill_router
    participant AM as AgentManager
    participant BI as BridgeInvoker
    participant BS as HapLotesBridgeServer
    participant PR as McpRouter（Cosmos）

    TUI->>MSR: McpMessage::CallTool(tool_name, agent_type, params)
    MSR->>AM: get_tool_location(tool_name)
    AM-->>MSR: ToolLocation

    alt ToolLocation::Cosmos
        MSR->>AM: invoke_tool(tool_name, agent_type, params)
        AM->>BI: 路由到正确的 agent
        BI->>BS: 通过 HapLotes 桥接转发
        BS->>PR: bridge.call(tool_name, params)
        PR-->>BS: 结果
        BS-->>BI: JSON-RPC 响应
        BI-->>AM: McpToolResult
        AM-->>MSR: McpToolResult
        MSR-->>TUI: McpMessage::ToolResponse
    else ToolLocation::Scepter
        MSR->>AM: 通过 HapLotes 网关路由
        AM->>AM: mcp_tools.invoke() 本地
        AM-->>TUI: 通过 WS 发送 McpMessage::ToolResponse
    end
```

### 关键路由逻辑

路由决策在 `mcp_skill_router.rs` 中发生：

1. 检查 `agent_manager.get_tool_location(tool_name)`
1. 如果 `ToolLocation::Cosmos` 且容器化模式激活：

   - 调用 `agent_manager.invoke_tool()`，通过 `BridgeInvoker` → HapLotes 桥接 → Cosmos 的 `McpRouter` 路由
   - Cosmos 的 `McpRouter` 在本地调度（skemma）或通过桥接回到 Scepter 调度远程 agent
   - 直接向 TUI 返回 `McpMessage::ToolResponse`

1. 否则：通过 HapLotes 网关路由到 agent 进程

## IV. CosmosConnector / 桥接架构

### HapLotes 桥接（当前）

HapLotes 桥接是 Scepter 和 Cosmos 容器之间的**唯一通信通道**。

```mermaid
flowchart LR
    subgraph Cosmos["Cosmos（容器）"]
        MR[McpRouter] -->|ToolSource::Local| SK[skemma Boa JS]
        MR -->|ToolSource::Bridge| BC[HapLotesBridgeClient]
    end

    subgraph Scepter["Scepter（主机）"]
        BS[HapLotesBridgeServer] --> BI[BridgeInvoker]
        BI --> AG1[Aporia]
        BI --> AG2[KaLos]
        BI --> AG3[...所有 agent]
    end

    BC -->|Unix 套接字 JSON-RPC| BS
```

### 连接池（CosmosConnector——Scepter 侧）

```mermaid
classDiagram
    class CosmosConnector {
        -connections: RwLock~HashMap~String, CosmosConnection~~
        +connect(instance_uuid, socket_path) Result
        +invoke_tool(instance_uuid, tool_name, params) Result~Value~
        +list_tools(instance_uuid) Result~Vec~String~~
        +disconnect(instance_uuid)
    }

    class CosmosConnection {
        -transport: Mutex~JsonRpcTransport~
    }

    class JsonRpcTransport {
        +send_request(request) Result~JsonRpcResponse~
    }

    CosmosConnector --> CosmosConnection
    CosmosConnection --> JsonRpcTransport
```

### JSON-RPC 协议

所有方法名称使用 `UnixMethod` 枚举以确保编译时类型安全：

| UnixMethod 变体 | 方向 | 参数 |
| --- | --- | --- |
| `UnixMethod::McpCall` | Scepter → Cosmos | `{ tool_name, parameters }` |
| `UnixMethod::McpListTools` | Scepter → Cosmos | 无 |
| `UnixMethod::ReplSnapshot` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::ReplRestore` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::BridgeCall` | Cosmos → Scepter | `{ tool_name, parameters }` |
| `UnixMethod::BridgeListTools` | Cosmos → Scepter | 无 |

### 响应格式

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## V. 容器生命周期

```mermaid
stateDiagram-v2
    [*] --> Pending: 技能链已启动
    Pending --> Creating: SnowflakeManager.create()
    Creating --> Starting: 容器运行中
    Starting --> Connected: CosmosConnector.connect()
    Connected --> Ready: 健康检查通过

    Ready --> Executing: invoke_tool()
    Executing --> Ready: 结果已返回

    Ready --> Stopping: 技能链完成
    Stopping --> Disconnected: CosmosConnector.disconnect()
    Disconnected --> [*]

    Creating --> Failed: 超时
    Starting --> Failed: 连接错误
    Failed --> [*]
```

### 容器 Agent

在 Cosmos 容器内部，仅 skemma 在本地运行（Boa JS 引擎）。所有其他 agent 工具通过 HapLotes 桥接路由回 Scepter：

| Agent | 角色 | 在 Cosmos 中？ |
| --- | --- | --- |
| SkeMma | 脚本执行（Boa JS） | **本地**（进程内） |
| Aporia | LLM 聊天 | 通过桥接 → Scepter |
| KaLos | 文件 I/O | 通过桥接 → Scepter |
| NeiKos | 容器管理 | 通过桥接 → Scepter |
| EleOs | Web 搜索 | 通过桥接 → Scepter |
| 所有其他 | 各种 | 通过桥接 → Scepter |

## VI. 统计行集成

### 显示格式

在 TUI `AgentDetailPage` 中，统计行显示：

```mermaid
flowchart LR
    BORDER["|"] --> TOK["1.2k tokens"] --> SEP1["|"] --> DUR["3.5s"] --> SEP2["|"] --> COSMOS["cosmos#042"] --> TIER["[T2]"]

    TOK -.->|"McpToolResult.token_usage"| SRC1["Token 使用量"]
    DUR -.->|"Instant::now()"| SRC2["耗时"]
    COSMOS -.->|"AgentIdManager"| SRC3["Agent 编号"]
    TIER -.->|"McpToolConfig.tier"| SRC4["模型层级"]
```

| 段 | 来源 |
| --- | --- |
| `1.2k tokens` | `McpToolResult.token_usage` |
| `3.5s` | 从 `Instant::now()` 计算的耗时 |
| `cosmos#042` | 来自 `AgentIdManager` 的 Agent 编号 |
| `[T2]` | 来自 `McpToolConfig.tier` 的模型层级 |

## VII. 错误处理

### 故障模式

```mermaid
flowchart TD
    Call[工具调用] --> Q1{容器在线？}
    Q1 -->|否| E1[错误：AGENT_OFFLINE]
    Q1 -->|是| Q2{套接字已连接？}
    Q2 -->|否| E2[错误：连接丢失]
    Q2 -->|是| Q3{工具存在？}
    Q3 -->|否| E3[错误：工具未找到]
    Q3 -->|是| Q4{执行成功？}
    Q4 -->|否| E4[错误：执行失败]
    Q4 -->|是| Result[返回结果]

    E1 --> Fallback[回退：尝试 Scepter 执行]
    E2 --> Retry[重试：重新连接套接字]
```

### 优雅降级

当容器不可用时，如果工具注册了本地实现，系统可以选择性地回退到 `Scepter` 本地执行。

## VIII. 未来扩展

| 功能 | 描述 | 优先级 |
| --- | --- | --- |
| 容器池化 | 跨技能链重用容器 | 中 |
| 健康监控 | 定期容器健康检查 | 高 |
| 资源限制 | 每容器的 CPU/内存限制 | 高 |
| 多容器工具 | 跨多个容器的工具 | 低 |
| 容器迁移 | 在主机之间移动运行中的容器 | 低 |
