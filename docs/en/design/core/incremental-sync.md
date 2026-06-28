+++
title = "Incremental Sync Architecture"
description = """A multi-client state incremental synchronization mechanism based on Automerge CRDT, supporting real-time incremental updates and full synchronization on connection/reconnection, covering all TUI panel"""
lang = "en"
category = "design"
subcategory = "core"
+++

# Incremental Sync Architecture

## Overview

A multi-client state incremental synchronization mechanism based on Automerge CRDT, supporting real-time incremental updates and full synchronization on connection/reconnection, covering all TUI panels.

## Architecture Diagram

```mermaid
flowchart TB
    subgraph Clients["TUI Clients (Multiple)"]
        C1["Client 1"]
        C2["Client 2"]
        C3["Client N"]
    end

    subgraph Server["Server"]
        SM["SyncManager<br/>Single State Tree"]
        BH["BroadcastHelper"]
        WS["WebSocket Broadcast"]
        REG["StateRegistry<br/>Full State"]
    end

    subgraph Storage["Automerge CRDT"]
        AD["AgentDoc<br/>per-agent"]
        V["Version Vectors"]
    end

    %% Full sync requests (pull mode)
    C1 -->|"On Connection"| WS
    C2 -->|"On Connection"| WS
    C3 -->|"On Connection"| WS

    WS -->|"RequestFullSnapshot"| BH
    BH -->|"list_agents"| SM
    SM -->|"AgentSnapshot"| BH
    BH -->|"broadcast"| WS
    WS -->|"AgentSnapshot"| C1
    WS -->|"AgentSnapshot"| C2
    WS -->|"AgentSnapshot"| C3

    %% Incremental updates (push mode)
    SM -->|"State Change"| BH
    BH -->|"update_agent"| SM
    SM -->|"Generate AgentPatch"| BH
    BH -->|"broadcast"| WS
    WS -->|"AgentPatch"| C1
    WS -->|"AgentPatch"| C2
    WS -->|"AgentPatch"| C3

    %% Automerge storage
    SM <-->|"agent_docs"| AD
    SM <-->|"version"| V

    style SM fill:#e1f5fe
    style BH fill:#fff3e0
    style WS fill:#f3e5f5
    style AD fill:#e8f5e9
    style REG fill:#fff9c4
```

## Sync Strategy Matrix

| Panel | Sync Method | Trigger | Frequency | Message Types |
| --- | --- | --- | --- | --- |
| **Agents Timeline** | Incremental + Full | Sync on Connection + Real-time Push | On Connection / Real-time | `AgentPatch` / `GlobalSnapshot` |
| **Containers** | Incremental + Full | Sync on Connection + Real-time Push | On Connection / Real-time | `ContainerPatch` / `GlobalSnapshot` |
| **Tasks** | Incremental + Full | Sync on Connection + Real-time Push | On Connection / Real-time | `TaskPatch` / `GlobalSnapshot` |
| **Models List** | Full | Client Active Request | When Opening Panel | `ModelsSnapshot` |
| **Providers Config** | Full | Client Active Request | When Opening Panel | `ProvidersSnapshot` |

## Message Flow

### Incremental Update Flow (Agents)

```mermaid
sequenceDiagram
    participant Agent as Agent Runtime
    participant SM as SyncManager
    participant BH as BroadcastHelper
    participant WS as WebSocket
    participant Client as TUI Client

    Agent->>SM: State Update
    SM->>SM: update_agent()
    SM->>SM: Generate AgentPatch
    SM->>BH: Return patch
    BH->>WS: broadcast(AgentPatch)
    WS->>Client: AgentPatch message
    Client->>Client: apply_agent_patch()
    Client->>Client: Update UI
```

### Full Sync Flow

```mermaid
sequenceDiagram
    participant Client as TUI Client
    participant WS as WebSocket
    participant BH as BroadcastHelper
    participant SM as SyncManager
    participant Registry as State Registry

    Note over Client: On Connection / Reconnection
    Client->>WS: RequestGlobalSnapshot
    WS->>BH: send_full_snapshot()
    BH->>Registry: list_agents()
    Registry-->>BH: Vec<AgentInfo>
    BH->>SM: create_snapshot(agents)
    SM-->>BH: AgentSnapshot
    BH->>WS: broadcast(AgentSnapshot)
    WS-->>Client: AgentSnapshot message
    Client->>Client: Replace local state
```

### Models List Sync Flow

```mermaid
sequenceDiagram
    participant Client as TUI Client
    participant WS as WebSocket
    participant Server as Server

    Note over Client: When opening Models panel
    Client->>WS: Request Models list
    WS->>Server: Query Models config
    Server-->>WS: Models list
    WS-->>Client: ModelsSnapshot message
    Client->>Client: Update Models panel
```

### Containers Full Sync Flow

```mermaid
sequenceDiagram
    participant Client as TUI Client
    participant WS as WebSocket
    participant Server as Server

    Note over Client: When opening Containers panel
    Client->>WS: RequestContainerSnapshot
    WS->>Server: Query Containers state
    Server-->>WS: ContainerSnapshot
    WS-->>Client: ContainerSnapshot message
    Client->>Client: Replace local Containers state
```

### Tasks Full Sync Flow

```mermaid
sequenceDiagram
    participant Client as TUI Client
    participant WS as WebSocket
    participant Server as Server

    Note over Client: When opening Tasks panel
    Client->>WS: RequestTasksSnapshot
    WS->>Server: Query Tasks state
    Server-->>WS: TasksSnapshot
    WS-->>Client: TasksSnapshot message
    Client->>Client: Replace local Tasks state
```

## Data Structures

### AgentPatch (Incremental Update)

```rust
pub struct AgentPatch {
    pub agent_id: String,
    pub version: u64,
    pub llm_working_changed: Option<bool>,
    pub work_status: Option<String>,
    pub current_model: Option<String>,
    pub token_usage_delta: Option<(u32, u32)>,
    pub token_usage_absolute: Option<(u32, u32)>,
    pub request_state: Option<RequestState>,
    pub cpu_usage: Option<f64>,
    pub memory_mb: Option<u64>,
}
```

### AgentSnapshot (Full Snapshot)

```rust
pub struct AgentSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
}
```

### GlobalSnapshot (Global Snapshot)

```rust
pub struct GlobalSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
    pub models: Vec<ModelInfo>,
    pub providers: Vec<ProviderInfo>,
    pub active_tasks: Vec<TaskInfo>,
}
```

### ModelsSnapshot (Models List)

```rust
pub struct ModelsSnapshot {
    pub models: Vec<ModelInfo>,
}
```

### ContainerPatch (Container State Incremental)

```rust
pub struct ContainerPatch {
    pub container_id: String,
    pub version: u64,
    pub status_changed: Option<String>,
    pub cpu_usage_changed: Option<f64>,
    pub memory_usage_changed: Option<u64>,
}
```

### ContainerSnapshot (Container State Full)

```rust
pub struct ContainerSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub containers: Vec<ContainerInfo>,
}
```

### TaskPatch (Task State Incremental)

```rust
pub struct TaskPatch {
    pub task_id: Uuid,
    pub version: u64,
    pub status_changed: Option<String>,
    pub progress_changed: Option<u8>,
}
```

### TasksSnapshot (Tasks State Full)

```rust
pub struct TasksSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub tasks: Vec<TaskInfo>,
}
```

## Sync Strategy

| Type | Direction | Trigger | Frequency |
| --- | --- | --- | --- |
| Agent Incremental Update | Server → Client | State Change | Real-time |
| Agent Full Sync | Server → Client | On Connection | On Connection / Reconnection |
| Containers Incremental | Server → Client | State Change | Real-time |
| Containers Full Sync | Server → Client | On Connection | On Connection / Reconnection |
| Tasks Incremental | Server → Client | State Change | Real-time |
| Tasks Full Sync | Server → Client | On Connection | On Connection / Reconnection |
| Models List | Client → Server | Active Request | When opening panel |
| Providers Config | Client → Server | Active Request | When opening panel |

## Key Features

- **Single State Tree**: Server maintains one `SyncManager`, all clients receive the same state updates
- **CRDT Conflict Resolution**: Automatic conflict resolution based on Automerge
- **Incremental Updates**: Only transmit changed fields to reduce network traffic
- **Eventual Consistency**: Full sync on connection guarantees eventual consistency
- **On-Demand Pull**: Models and Providers are requested on-demand when opening their panels to avoid unnecessary network transmission
- **Home Page Sync**: Agents, Containers, and Tasks are synced on connection since they're visible on the home page

## Implementation Status

- ✅ Agents incremental/full sync
- ✅ Models list sync
- ✅ Providers config sync
- ✅ Containers incremental/full sync
- ✅ Tasks incremental/full sync
- ✅ State persistence (/tmp storage, reload on restart)
