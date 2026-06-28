+++
title = "ADR-004: 60+ Crate Layered Workspace Architecture"
description = """Date: 2026-03"""
lang = "en"
category = "design"
subcategory = "core"
+++

# ADR-004: 60+ Crate Layered Workspace Architecture

**Date**: 2026-03
**Status**: Accepted

## Context

Entelecheia started with a monolithic `packages/shared` crate (38K lines, 187 `.rs` files) that contained all shared infrastructure: types, MCP protocol, LLM providers, container management, database, security, configuration, and more. As the project grew to 12 agents + 1 domain agent + 3 binary packages, several problems emerged:

1. **Compile times**: Any change to `shared` required recompiling all 187 files, even if only one struct was modified.
1. **Dependency pollution**: Agent crates that only needed MCP types were forced to transitively depend on database drivers, container runtimes, and LLM providers.
1. **Unclear ownership**: With 187 files in one crate, it was unclear which module "owned" which functionality, making refactoring risky.
1. **Feature flag explosion**: Conditional compilation via Cargo features was used to avoid pulling in unnecessary dependencies, but this led to combinatorial explosion in test configurations.

## Decision

Decompose the monolithic `packages/shared` into **37 focused sub-crates** organized in **6 dependency layers** (L0 through L5), following a strict dependency direction:

```text
L0 (leaf) → L1 → L2 → L3 → L4 → L5 → consumers (scepter, agents, tui)
```

**Layer definitions:**

| Layer | Crates | Rule |
| --- | --- | --- |
| **L0** | core, logging, macros | Zero internal dependencies on other entelecheia crates |
| **L1** | domain_enums, mcp_types, text, concurrent | Depend only on L0 |
| **L2** | config, agent_registry, state_types | Depend on L0-L1 |
| **L3** | domain_agent, container, agent_lifecycle, agent_runtime, thread_types, toolchain, infra_utils | Depend on L0-L2 |
| **L4** | state_sync, domain_skills, hooks, domain_auth, container_runtime, skills_permissions, timeline, iepl | Depend on L0-L3 |
| **L5** | llm_provider, prompt, custom_agent, storage, infra_jsonrpc, infra_services, e2e_events, adapter, plugin_host, rag, embedding, security_policy | Depend on L0-L4 |

All internal dependency declarations use `workspace = true` for version consistency. No thin aggregator crate exists — consumers import directly from individual sub-crates.

## Consequences

### Positive

- **Incremental compilation**: A change to `shared-core` (L0) still propagates, but a change to `shared-security-policy` (L5) only recompiles that crate and its direct consumers. Build times improved significantly.
- **Clear ownership boundaries**: Each crate has a focused responsibility. Code review scope is naturally bounded by crate boundaries.
- **Dependency isolation**: Agent crates import only the shared crates they need. SkeMma doesn't pull in database drivers. EleOs doesn't pull in container runtimes.
- **Circular dependency prevention**: The layered architecture makes it structurally impossible to create circular dependencies — L3 crates cannot depend on L5 crates.
- **Testable in isolation**: Each crate's tests run independently, without requiring the full workspace's dependency tree.

### Negative

- **Workspace management overhead**: 60+ crates in a single workspace means more `Cargo.toml` files to maintain, more `[dependencies]` sections to update on version bumps, and more careful dependency declaration.
- **Cross-crate refactoring is harder**: Moving a type from L2 to L3 requires updating all L2 consumers and verifying no L3+ crate accidentally depends on the moved type through the old location.
- **Crate naming verbosity**: Internal crate names use the `_shared_*` prefix convention (e.g., `_shared_domain_skills_permissions`), which is verbose but necessary for workspace clarity.
- **Potential over-decomposition**: Some crates (e.g., `shared-text` with ~200 lines) may not justify their own crate overhead. The decomposition followed a "separate if it might grow" philosophy rather than strict necessity.

### Trade-off Accepted

**Management complexity for compile-time and architectural clarity.** A 37-crate decomposition of `shared` is on the aggressive end of Rust workspace design. A middle ground (10-15 crates) would have been simpler to manage. However, given the project's broad surface area (26 LLM providers, 2 container runtimes, 12 agents, full security pipeline, database, IEPL), fine-grained decomposition ensures that each piece can evolve independently. The `workspace = true` pattern mitigates version management overhead.
