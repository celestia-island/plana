+++
title = "Entelecheia Security Architecture"
description = """> Comprehensive defense-in-depth model for the Entelecheia Multi-Agent Orchestration Platform."""
lang = "en"
category = "design"
subcategory = "core"
+++

# Entelecheia Security Architecture

> Comprehensive defense-in-depth model for the Entelecheia Multi-Agent Orchestration Platform.

## Overview

Entelecheia implements a **defense-in-depth security architecture** spanning 14 independently testable security layers — from hardware-level container isolation to LLM-facing tool permission gates. Unlike traditional agent frameworks that expose all tools directly to the LLM, Entelecheia's **Exec-Only Microkernel** design means the LLM sees only 3 primitive tools (`exec`, `write_to_var`, `write_to_var_json`), while 148 MCP tools are dispatched through a typed IEPL pipeline with multi-layer authorization.

## Security Layer Index

| # | Layer | Crate(s) | Threat Mitigated |
| --- | --- | --- | --- |
| 1 | Exec-Only Microkernel | `scepter`, `mcp_types` | Unrestricted tool access by LLM |
| 2 | Dual-Authorization Permission Gate | `security_policy` | Unauthorized MCP tool invocation |
| 3 | Trust-Level Skill Authorization | `domain_skills_permissions` | Privilege escalation via skill chaining |
| 4 | Container Isolation (Outer) | `container` (Docker/Podman) | Host compromise from agent code |
| 5 | OCI Sandbox (Inner) | `container_runtime` (Youki/libcontainer) | Container escape |
| 6 | RBAC Access Control | `domain_auth`, shittim-chest `rbac` | Unauthorized API access |
| 7 | JWT Authentication | shittim-chest `auth` (HS256) | Session hijacking, replay attacks |
| 8 | API Key Encryption | `aporia` (AES-256-GCM) | Credential leakage at rest |
| 9 | Security Sentinel | `orexis` (OreXis agent) | Malicious code execution, compliance violations |
| 10 | IEPL Type-Safe Pipeline | `iepl`, `iepl_engine`, `skemma` | Injection via untyped tool calls |
| 11 | Provider Registry Whitelist | `config/registries.toml` | Supply chain attacks via untrusted packages |
| 12 | Prompt Injection Defense | IEPL sandbox boundary | LLM prompt injection via tool output |
| 13 | Rate Limiting | shittim-chest `channel/rate_limit` | DoS, resource exhaustion |
| 14 | Audit Trail | `orexis`, `timeline` | Post-incident forensics, accountability |

---

## Layer 1: Exec-Only Microkernel

**Crates:** `scepter`, `mcp_types`
**Design Philosophy:** Minimize LLM attack surface

The LLM operates in an **exec-only sandbox** where it can invoke only three primitive operations:

| Tool | Purpose | Parameters |
| --- | --- | --- |
| `exec` | Execute a script string | JavaScript code (transpiled from TypeScript via IEPL) |
| `write_to_var` | Store a string value | Variable name + value |
| `write_to_var_json` | Store a JSON value | Variable name + JSON value |

All 148 MCP tools (file operations, container management, device control, web search, etc.) are **invisible to the LLM**. They are invoked indirectly through the IEPL pipeline when the LLM's `exec` calls ES module imports (e.g., `import { file_read } from 'kalos'`).

**Threat model:** Even if the LLM is compromised via prompt injection, it cannot directly invoke dangerous tools like `container_destroy` or `ssh_exec`. The IEPL pipeline enforces type checking and permission verification before any tool executes.

**Implementation:** `packages/shared/mcp_types/src/` defines the microkernel IPC types. The `exec` handler in `packages/cosmos/` transpiles and executes the script via the Boa engine, with tool calls routed through `skemma`'s `McpRouter`.

---

## Layer 2: Dual-Authorization Permission Gate

**Crate:** `security_policy` (5,772 lines)

Every MCP tool declares its access requirements via a **permission level** enum. Every skill (IEPL script) declares the permission level it needs per tool. Both must agree for a call to proceed.

```rust
pub enum PermissionLevel {
    /// Read-only operations (file_read, list_dir, etc.)
    Read,
    /// Write operations within the workspace (file_write, exec_script)
    Write,
    /// Operations affecting external systems (ssh_exec, container_deploy)
    System,
    /// Operations with irreversible consequences (container_destroy, device_reboot)
    Destructive,
}
```

**Authorization flow:**

1. Skill declares: "I need `System` access to `ssh_exec`"
1. Tool declares: "I require `System` permission"
1. Permission gate checks: `skill_level >= tool_requirement` AND `skill is explicitly granted this tool`
1. If either check fails: call is blocked, logged, and reported to OreXis sentinel

**Implementation:** `packages/shared/security_policy/src/` — 107 test annotations, 4 tokio tests.

---

## Layer 3: Trust-Level Skill Authorization

**Crate:** `domain_skills_permissions` (1,776 lines)

Skills are classified into **trust levels** that determine their default permission scope:

| Trust Level | Description | Default Permissions |
| --- | --- | --- |
| `Builtin` | Ships with the platform | Full tool access |
| `Verified` | Reviewed and signed by maintainers | Read + Write |
| `Community` | Submitted by users | Read only |
| `Untrusted` | Dynamically loaded | No tool access (exec only) |

Each skill's trust level is verified at load time and cached. Attempts to escalate trust level are logged as security events.

---

## Layer 4: Container Isolation (Outer Ring)

**Crate:** `container` (5,742 lines)

Every agent execution occurs inside a **Docker or Podman container** with:

- Network namespace isolation
- Read-only root filesystem (except workspace mount)
- Seccomp profile restricting syscalls
- Resource limits (CPU, memory, PID count)
- No access to host Docker socket

**Implementation:** `packages/shared/container/src/` — 74 test annotations, 12 tokio tests. Supports both Docker (via Bollard API) and Podman.

---

## Layer 5: OCI Sandbox (Inner Ring)

**Crate:** `container_runtime` (3,645 lines)

Inside the Docker container, Entelecheia runs a **second isolation layer** using Youki/libcontainer — a daemonless, rootless OCI-compliant container runtime. This provides:

- Rootless execution (no privilege escalation possible)
- Namespace isolation independent of Docker
- Cgroup v2 enforcement
- Seccomp filter (deny-by-default)

**Why two layers?** Docker provides coarse-grained isolation (network, filesystem). Youki provides fine-grained syscall filtering and resource accounting. If Docker is compromised, the Youki sandbox still contains the agent.

---

## Layer 6: RBAC Access Control

**Crates:** `domain_auth` (380 lines), shittim-chest `rbac` (1,736 lines)

Role-based access control governing all API operations:

- **Groups:** Users belong to groups; groups have grants
- **Grants:** Fine-grained permissions (read/write/admin per resource type)
- **Workspace isolation:** Users can only access workspaces they're members of
- **Cross-workspace operations:** Require explicit admin grants

---

## Layer 7: JWT Authentication

**Module:** shittim-chest `auth/jwt.rs` (264 lines)

- **Algorithm:** HS256 (HMAC-SHA256)
- **Access tokens:** Short-lived (configurable, default 15 min)
- **Refresh tokens:** Longer-lived with rotation on use
- **Nonce-based CSRF protection** for browser clients
- **Rate limiting** on auth endpoints (GCRA algorithm)

---

## Layer 8: API Key Encryption

**Crate:** `aporia` (5,802 lines)

All LLM provider API keys are encrypted at rest using **AES-256-GCM** with:

- Unique nonce per encryption operation
- Key derived from a master secret (environment-configured)
- Zeroization of plaintext keys from memory after use
- Key rotation support

---

## Layer 9: Security Sentinel (OreXis)

**Crate:** `orexis` (5,239 lines) — the "immune system" agent

OreXis is a Layer-1 Agent that:

- **Audits code** for security vulnerabilities and license compliance
- **Inspects tool calls** against registered security policies
- **Blocks/unblocks** any agent's tools by pattern
- **Monitors** agent behavior for anomalous patterns

MCP tools (24): `standard_check`, `compliance_report`, `audit_alignment`, `audit_legality`, `agent_integrity`, `security_audit`, `tool_block`, `tool_unblock`, `policy_register`, `policy_list`, etc.

---

## Layer 10: IEPL Type-Safe Pipeline

**Crates:** `iepl` (2,670 lines), `iepl_engine` (1,228 lines), `skemma` (7,960 lines)

The **Entelecheia Plugin Language** (IEPL) pipeline ensures type safety between LLM-generated code and native tool dispatch:

1. LLM generates TypeScript code using ES module imports
1. **SWC** transpiles TypeScript → JavaScript (syntax validation)
1. **Boa engine** executes JavaScript in a sandboxed context
1. ES module imports are resolved to `__native_dispatch` calls
1. Each dispatch is routed through `McpRouter` with full type checking

**Threat mitigated:** Injection attacks via untyped tool calls (common in Python-based agent frameworks where tool schemas are validated only at runtime).

---

## Layer 11: Provider Registry Whitelist

**File:** `configs/registries.toml` (337 lines)

Entelecheia maintains a **hardcoded whitelist** of trusted package registries across 15 ecosystems:

crates.io, PyPI, npm, Go modules, Docker Hub, Maven Central, NuGet, RubyGems, Hackage, Alpine APK, Debian APT, GitHub, GitLab, `HuggingFace`, PyTorch.

Any package import from a non-whitelisted registry is **blocked at the container level** before execution.

---

## Layer 12: Prompt Injection Defense

**Mechanism:** IEPL sandbox boundary

The LLM's `exec` output is executed in an **isolated Boa JS context** with no access to:

- The host filesystem
- Network sockets
- Environment variables
- Other agents' state

Tool outputs returned to the LLM are **sanitized** — binary data is base64-encoded, excessive output is truncated, and potential prompt injection patterns in tool results are flagged by OreXis.

---

## Layer 13: Rate Limiting

**Module:** shittim-chest `channel/rate_limit.rs` (118 lines)

Per-user, per-channel rate limiting using the **GCRA (Generic Cell Rate Algorithm)**:

- Configurable burst size and sustained rate
- Per-user DashMap for O(1) lookup
- Automatic backoff on limit exceeded
- Separate limits for API calls, message sends, and tool invocations

---

## Layer 14: Audit Trail

**Crates:** `orexis`, `timeline` (3,096 lines)

Every tool invocation, agent decision, and security event is:

1. Recorded in the **timeline** with full context (agent badge, skill name, parameters, result)
1. Hash-linked to previous events for tamper detection
1. Persisted to PostgreSQL with configurable retention
1. Queryable via the CLI (`entelecheia-cli trace-chain <badge>`)

---

## Security Comparison with Other Frameworks

| Feature | Entelecheia | OpenFANG | LangChain | Claude Code |
| --- |  ---  |  ---  |  ---  |  ---  |
| LLM-visible tools | **3 (exec-only)** | 53 (all visible) | All visible | 33 (all visible) |
| Container isolation | **Dual-layer** (Docker + Youki) | WASM only | None | OS-level (Seatbelt/Landlock) |
| Tool permission model | **Dual-authorization** | RBAC | None | None |
| Code audit agent | **OreXis (24 tools)** | Loop guard | None | None |
| Type-safe dispatch | **IEPL pipeline** | Direct function call | Direct function call | Direct function call |
| Package whitelist | **15 registries** | None | None | None |
| Audit trail | Hash-linked timeline | Merkle hash-chain | None | None |

---

## Threat Model

### Out of Scope

- Physical access to host machines
- Compromised Docker/Podman daemon (assumed trusted)
- Kernel exploits (mitigated but not prevented by user-space isolation)
- Supply chain attacks on Rust crate dependencies (partially mitigated by `cargo-deny`)

### Accepted Risks

- Boa JS engine vulnerabilities (sandboxed within container)
- LLM provider outages (no fallback execution path)
- PostgreSQL data corruption (mitigated by backups, not prevented)

---

## Reporting Vulnerabilities

See [SECURITY.md](../SECURITY.md) for the vulnerability reporting process.

## License

This security architecture is part of Entelecheia, licensed under [BUSL-1.1](../LICENSE).
