+++
title = "ADR-005: Container-Sandboxed Agent Execution with COSMOS"
description = """Date: 2026-02"""
lang = "en"
category = "design"
subcategory = "core"
+++

# ADR-005: Container-Sandboxed Agent Execution with COSMOS

**Date**: 2026-02
**Status**: Accepted

## Context

In a multi-agent system where agents execute LLM-generated code, isolation between agents is critical for:

1. **Security**: Untrusted LLM output should not be able to access another agent's memory, files, or network connections.
1. **State isolation**: Each agent's REPL state (JavaScript variables, bindings, snapshots) must be independent.
1. **Resource control**: A misbehaving agent should not consume unlimited CPU, memory, or PIDs.
1. **Reproducibility**: Agent state should be snapshotable and restorable for debugging and rollback.
1. **Fork/merge workflows**: The system needs to support branching agent execution (fork) and merging results back (merge), similar to git branching.

Several isolation approaches were evaluated:

| Approach | Isolation Strength | Resource Control | Snapshot/Fork | Overhead |
| --- | --- | --- | --- | --- |
| **Container per agent (Docker/OCI)** | Strong (kernel-level) | Full (cgroups, seccomp, capabilities) | Native (commit/snapshot) | Moderate (~100ms startup, ~50MB per container) |
| **Process per agent** | Moderate (UID/seccomp) | Partial (rlimit) | Manual (serialize state) | Low |
| **Thread per agent** | Weak (shared memory) | Minimal | Manual | Minimal |
| **WASM sandbox per agent** | Strong (linear memory) | Good (gas metering) | Manual | Low |
| **Boa context per agent** | Moderate (JS sandbox) | Limited | Built-in (namespace serialization) | Minimal |

## Decision

We chose a **two-layer container architecture** with **COSMOS** as the init process inside each agent's container:

**Outer layer (orchestration infrastructure):**

- Docker/Podman via Bollard for infrastructure containers (PostgreSQL, Scepter daemon).
- Full orchestration capabilities: networking, volumes, health checks, compose.

**Inner layer (agent sandboxes):**

- Youki/libcontainer (default) or Docker for per-agent COSMOS containers.
- Each agent gets its own container with COSMOS as PID 1.
- COSMOS is the **front-end process** that mediates all interactions — it provides the JSON-RPC Unix socket server, the Boa JS REPL, the MCP router, and the HapLotes bridge connection back to Scepter.

**Why COSMOS as the mandatory intermediary:**

All interactions with a containerized agent must go through COSMOS. Direct container manipulation (e.g., `docker exec` into a container) bypasses the security model, state management, and audit trail. COSMOS provides:

1. **Tool dispatch mediation**: The `McpRouter` enforces allowlists, dual-authorization, and trust levels before any tool reaches the agent.
1. **State persistence**: Double-buffered snapshot system ensures REPL state survives crashes.
1. **Bridge communication**: The HapLotes bridge connects COSMOS back to Scepter for inter-agent coordination.
1. **Security enforcement**: Seccomp profiles, egress policies, and capability restrictions are applied at container creation and enforced by the kernel.

**Why Youki/libcontainer for inner sandboxes:**

- Rootless and daemonless — no Docker daemon required for agent sandboxes.
- OCI-compliant — standard `config.json` spec, compatible with OCI tooling.
- Fast overlay-based rootfs — snapshot and fork operations copy only changed files.
- Lower overhead than Docker for ephemeral containers.

## Consequences

### Positive

- **Strong isolation via kernel enforcement**: cgroups (CPU/memory/PID limits), seccomp (syscall filtering), capabilities (`cap_drop`=ALL), namespaces (PID/network/mount isolation).
- **Native fork/merge**: Container commit creates an image snapshot; new containers can be created from the snapshot. Overlay filesystems track only changed files.
- **Resource limits per agent**: 512MB memory, 1 CPU, 100 PIDs by default, configurable per container.
- **Audit trail**: All tool calls pass through COSMOS's MCP router, which logs every dispatch for OreXis security auditing.
- **Crash containment**: A Boa panic or agent bug is confined to its container. Other agents and Scepter continue operating.
- **Youki for lightweight sandboxes**: Inner containers start faster and consume fewer resources than full Docker containers.

### Negative

- **Complexity of COSMOS as PID 1**: COSMOS must handle signal forwarding, zombie reaping, and clean shutdown as the container's init process. This adds responsibility that a normal application doesn't have.
- **Container startup latency**: Each agent container requires ~100ms-1s to start (depending on runtime). This is slower than process-based or thread-based isolation.
- **Resource overhead**: Each COSMOS container consumes ~50-100MB of memory for the Boa runtime, JS heap, and OS overhead. With 9 containerized agents, this adds ~0.5-1GB of baseline memory.
- **Testing complexity**: Testing agent behavior requires running actual containers with COSMOS, which means tests need Docker or Youki available. The snowflake test pattern (building the entelecheia image, running a COSMOS container, connecting via Unix socket) is more complex than unit testing.
- **Two runtimes to maintain**: Both Docker/Bollard and Youki/libcontainer code paths must be maintained and tested.

### Trade-off Accepted

**Resource overhead for security and isolation guarantees.** A process-per-agent model would use less memory and start faster, but would not provide kernel-level isolation between agents. In a system where agents execute untrusted LLM-generated code, the security guarantee of container isolation is worth the resource cost. The COSMOS-as-mandatory-intermediary design ensures that even if an attacker gains code execution inside a container, they cannot bypass the security model by operating outside of COSMOS's mediation.
