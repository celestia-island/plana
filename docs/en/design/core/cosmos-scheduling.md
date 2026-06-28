+++
title = "Cosmos Container Scheduling and Token Routing Design"
description = """This document describes the Cosmos container scheduling architecture: how MCP tools marked with `ToolLocation::Cosmos` are routed through unix-socket JSON-RPC to their corresponding containers, and ho"""
lang = "en"
category = "design"
subcategory = "core"
+++

# Cosmos Container Scheduling and Token Routing Design

## Overview

This document describes the Cosmos container scheduling architecture: how MCP tools marked with `ToolLocation::Cosmos` are routed through unix-socket JSON-RPC to their corresponding containers, and how the token (agent number) system ties into container identity and routing.

## I. Tool Location Model

### Dual Execution Environment

```mermaid
flowchart LR
    subgraph Scepter["Scepter (Central Process)"]
        A1[LLM Calls]
        A2[RAG Queries]
        A3[Task Management]
        A4[Credential Storage]
    end

    subgraph Cosmos["Cosmos (Container Per Agent)"]
        P1[File System Access]
        P2[Script Execution]
        P3[Hardware Access]
        P4[REPL Sessions]
    end

    Scepter -->|ToolLocation::Scepter| Local[Local Invoke]
    Cosmos -->|ToolLocation::Cosmos| Socket[Unix Socket RPC]
```

### ToolLocation Enum

| Variant | Execution Site | Transport |
| --- | --- | --- |
| `Scepter` (default) | In-process via `McpToolInvoker` | Direct function call |
| `Cosmos` | In container via `CosmosConnector` | Unix socket JSON-RPC |

### Location Decision Criteria

```mermaid
flowchart TD
    Tool[MCP Tool] --> Q1{Needs Container Resources?}
    Q1 -->|Yes: file system, scripts, hardware| Cosmos[ToolLocation::Cosmos]
    Q1 -->|No: LLM, RAG, state management| Scepter[ToolLocation::Scepter]
```

Tools that require container resources (file system, script execution, hardware access) are marked `Cosmos`. Centralized services (LLM, RAG, task management, human interaction) remain `Scepter`.

## II. Token System and Container Identity

### Agent Number Allocation

```mermaid
sequenceDiagram
    participant SM as SkillChain Manager
    participant AIM as AgentIdManager
    participant SC as SnowflakeContainer
    participant PC as CosmosConnector

    SM->>AIM: Request agent number
    AIM-->>SM: Assign 000-999 token
    SM->>SC: Create container for token
    SC-->>SM: Container UUID + socket path
    SM->>PC: connect(UUID, socket_path)
    PC-->>SM: Connection established
```

### Token Properties

| Property | Description |
| --- | --- |
| Format | Three-digit number: `000`-`999` |
| Allocator | `AgentIdManager` in skill chain |
| Binding | One token per skill chain panel |
| Display | Shown in TUI stats line as `cosmos#NNN` |
| Persistence | Survives across agent restarts |

## III. Request Routing Flow

### TUI-originated MCP Call

```mermaid
sequenceDiagram
    participant TUI as TUI Client
    participant MSR as mcp_skill_router
    participant AM as AgentManager
    participant BI as BridgeInvoker
    participant BS as HapLotesBridgeServer
    participant PR as McpRouter (Cosmos)

    TUI->>MSR: McpMessage::CallTool(tool_name, agent_type, params)
    MSR->>AM: get_tool_location(tool_name)
    AM-->>MSR: ToolLocation

    alt ToolLocation::Cosmos
        MSR->>AM: invoke_tool(tool_name, agent_type, params)
        AM->>BI: route to correct agent
        BI->>BS: forward via HapLotes bridge
        BS->>PR: bridge.call(tool_name, params)
        PR-->>BS: result
        BS-->>BI: JSON-RPC response
        BI-->>AM: McpToolResult
        AM-->>MSR: McpToolResult
        MSR-->>TUI: McpMessage::ToolResponse
    else ToolLocation::Scepter
        MSR->>AM: route through HapLotes gateway
        AM->>AM: mcp_tools.invoke() locally
        AM-->>TUI: McpMessage::ToolResponse via WS
    end
```

### Key Routing Logic

The routing decision happens in `mcp_skill_router.rs`:

1. Check `agent_manager.get_tool_location(tool_name)`
1. If `ToolLocation::Cosmos` and containerized mode active:

   - Call `agent_manager.invoke_tool()` which routes through `BridgeInvoker` → HapLotes bridge → Cosmos's `McpRouter`
   - Cosmos's `McpRouter` dispatches locally (skemma) or back to Scepter via bridge for remote agents
   - Return `McpMessage::ToolResponse` directly to TUI

1. Otherwise: route through HapLotes gateway to the agent process

## IV. CosmosConnector / Bridge Architecture

### HapLotes Bridge (Current)

The HapLotes bridge is the **sole communication channel** between Scepter and Cosmos containers.

```mermaid
flowchart LR
    subgraph Cosmos["Cosmos (Container)"]
        MR[McpRouter] -->|ToolSource::Local| SK[skemma Boa JS]
        MR -->|ToolSource::Bridge| BC[HapLotesBridgeClient]
    end

    subgraph Scepter["Scepter (Host)"]
        BS[HapLotesBridgeServer] --> BI[BridgeInvoker]
        BI --> AG1[Aporia]
        BI --> AG2[KaLos]
        BI --> AG3[...all agents]
    end

    BC -->|Unix Socket JSON-RPC| BS
```

### Connection Pool (CosmosConnector — Scepter-side)

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

### JSON-RPC Protocol

All method names use the `UnixMethod` enum for compile-time type safety:

| UnixMethod Variant | Direction | Parameters |
| --- | --- | --- |
| `UnixMethod::McpCall` | Scepter → Cosmos | `{ tool_name, parameters }` |
| `UnixMethod::McpListTools` | Scepter → Cosmos | None |
| `UnixMethod::ReplSnapshot` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::ReplRestore` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::BridgeCall` | Cosmos → Scepter | `{ tool_name, parameters }` |
| `UnixMethod::BridgeListTools` | Cosmos → Scepter | None |

### Response Format

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## V. Container Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Pending: Skill Chain Started
    Pending --> Creating: SnowflakeManager.create()
    Creating --> Starting: Container Running
    Starting --> Connected: CosmosConnector.connect()
    Connected --> Ready: Health Check Pass

    Ready --> Executing: invoke_tool()
    Executing --> Ready: Result Returned

    Ready --> Stopping: Skill Chain Complete
    Stopping --> Disconnected: CosmosConnector.disconnect()
    Disconnected --> [*]

    Creating --> Failed: Timeout
    Starting --> Failed: Connection Error
    Failed --> [*]
```

### Container Agents

Inside Cosmos containers, only skemma runs locally (Boa JS engine). All other agent tools route through the HapLotes bridge back to Scepter:

| Agent | Role | In Cosmos? |
| --- | --- | --- |
| SkeMma | Script execution (Boa JS) | **Local** (in-process) |
| Aporia | LLM chat | Via bridge → Scepter |
| KaLos | File I/O | Via bridge → Scepter |
| NeiKos | Container management | Via bridge → Scepter |
| EleOs | Web search | Via bridge → Scepter |
| All others | Various | Via bridge → Scepter |

## VI. Stats Line Integration

### Display Format

In the TUI `AgentDetailPage`, the stats line shows:

```mermaid
flowchart LR
    BORDER["|"] --> TOK["1.2k tokens"] --> SEP1["|"] --> DUR["3.5s"] --> SEP2["|"] --> COSMOS["cosmos#042"] --> TIER["[T2]"]

    TOK -.->|"McpToolResult.token_usage"| SRC1["Token Usage"]
    DUR -.->|"Instant::now()"| SRC2["Duration"]
    COSMOS -.->|"AgentIdManager"| SRC3["Agent Number"]
    TIER -.->|"McpToolConfig.tier"| SRC4["Model Tier"]
```

| Segment | Source |
| --- | --- |
| `1.2k tokens` | `McpToolResult.token_usage` |
| `3.5s` | Duration from `Instant::now()` |
| `cosmos#042` | Agent number from `AgentIdManager` |
| `[T2]` | Model tier from `McpToolConfig.tier` |

## VII. Error Handling

### Failure Modes

```mermaid
flowchart TD
    Call[Tool Call] --> Q1{Container Online?}
    Q1 -->|No| E1[Error: AGENT_OFFLINE]
    Q1 -->|Yes| Q2{Socket Connected?}
    Q2 -->|No| E2[Error: Connection Lost]
    Q2 -->|Yes| Q3{Tool Exists?}
    Q3 -->|No| E3[Error: Tool Not Found]
    Q3 -->|Yes| Q4{Execution Success?}
    Q4 -->|No| E4[Error: Execution Failed]
    Q4 -->|Yes| Result[Return Result]

    E1 --> Fallback[Fallback: Try Scepter execution]
    E2 --> Retry[Retry: Reconnect socket]
```

### Graceful Degradation

When container is unavailable, the system can optionally fall back to `Scepter`-local execution if the tool has a local implementation registered.

## VIII. Future Extensions

| Feature | Description | Priority |
| --- | --- | --- |
| Container pooling | Reuse containers across skill chains | Medium |
| Health monitoring | Periodic container health checks | High |
| Resource limits | CPU/memory limits per container | High |
| Multi-container tools | Tools spanning multiple containers | Low |
| Container migration | Move running containers between hosts | Low |
