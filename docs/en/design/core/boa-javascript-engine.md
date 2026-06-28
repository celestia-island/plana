+++
title = "ADR-002: Boa as Embedded JavaScript Engine"
description = """Date: 2026-02"""
lang = "en"
category = "design"
subcategory = "core"
+++

# ADR-002: Boa as Embedded JavaScript Engine

**Date**: 2026-02
**Status**: Accepted

## Context

The IEPL (Interactive Execution Pipeline Layer) requires a JavaScript runtime to execute LLM-generated code inside each COSMOS container. This runtime must:

1. Be **embeddable** within a Rust application — it will run as the init process (PID 1) inside lightweight containers.
1. Be **safe and sandboxed** — LLM-generated code is untrusted by nature.
1. Support **ES module imports** for the tool dispatch mechanism (e.g., `import { file_read } from 'agent'`).
1. Have **minimal startup overhead** — containers are ephemeral and must be ready quickly.
1. Be **cross-platform** and easy to compile for the container target.

Several JavaScript/TypeScript runtimes were evaluated:

| Runtime | Language | Embeddable in Rust | Sandbox Control | Startup Speed | ES Module Support |
| --- | --- | --- | --- | --- | --- |
| **Boa** | Pure Rust | Native (Rust crate) | Full (host controls everything) | Fast (<10ms) | Partial (enough for IEPL) |
| **Deno** | Rust + V8 | Possible via FFI | Limited (V8 isolates) | Slow (~100ms) | Full |
| **QuickJS** | C | Via FFI/bindings | Moderate | Fast | Partial |
| **V8** | C++ | Via FFI (v8 crate) | Limited | Slow | Full |
| **wasmoon** | C (Lua) | Via FFI | Good | Fast | N/A (Lua) |
| **rquickjs** | C (QuickJS) | Via FFI | Moderate | Fast | Partial |

## Decision

We chose **Boa Engine** (v0.21) as the embedded JavaScript runtime.

**Primary reasons:**

1. **Pure Rust — zero FFI overhead.** Boa is written entirely in Rust, which means it compiles natively into the COSMOS binary with no C/C++ dependency chain. This eliminates an entire class of FFI-related security vulnerabilities, build complexity, and cross-compilation headaches. In the COSMOS container context where we control the init process, this is a critical advantage.

1. **"Invoke and use" simplicity.** Boa is designed as a library-first engine. It can be instantiated, configured with host functions, and executed in a few lines of Rust code. There is no separate process to manage, no IPC bridge to maintain, and no event loop complexity. The `JsReplHandle` in COSMOS creates a dedicated OS thread for the Boa runtime and communicates via standard Rust channels — a clean, composable architecture.

1. **Security is the top priority, not performance.** In the COSMOS sandbox, every millisecond of JS execution time is not on the critical path — the bottleneck is always the LLM inference round-trip. What matters is that the runtime gives us **complete control** over what the executed code can do. Boa's host function registration API lets us precisely define which functions are available (only the MCP tool dispatch functions), with no escape hatches. The AST security validator (which blocks `eval`, `require`, `process`, etc.) operates on the Boa AST, giving us a native Rust guarantee of enforcement.

1. **Sufficient ES module support for IEPL.** The IEPL pipeline uses a simulated module system — ES module imports are resolved at the namespace builder level, not by a real module loader. Boa's capabilities are more than sufficient for this pattern. We don't need a full Node.js-compatible module resolution algorithm.

## Consequences

### Positive

- **No C/C++ build dependency** — COSMOS compiles cleanly with just `cargo build`, no system-level library requirements.
- **Full sandboxing control** — Every function available to the executed JS code is explicitly registered by the host. No default I/O, network, or filesystem access.
- **Tight integration with Rust types** — `boa_gc::Trace` trait implementation for custom host objects, native `serde_json` interop, zero-copy where possible.
- **Crash safety** — Boa panics are caught by the OS thread boundary, preventing the COSMOS process from crashing due to malformed JS code.
- **Small binary footprint** — Compared to V8-based solutions, Boa adds significantly less to the COSMOS binary size.

### Negative

- **JavaScript conformance is incomplete** — Boa does not fully implement ECMAScript 2024+. Some advanced features (e.g., `WeakRef`, `FinalizationRegistry`, full `Promise` integration, `async/await` in all contexts) may have limitations or missing implementations.
- **Performance is not competitive with V8/`SpiderMonkey`** — Boa's interpreter is significantly slower than JIT-compiled engines. For CPU-intensive workloads (large data processing, complex algorithms), this matters. However, in the IEPL context, JS code is primarily orchestration glue calling MCP tools, not computation.
- **Ecosystem is smaller** — Boa has fewer contributors and less battle-testing than V8 or QuickJS. Bugs may take longer to fix upstream.
- **No `eval` or dynamic code generation** — By design (and enforced by the AST validator), dynamic code evaluation is blocked. This limits certain meta-programming patterns but is acceptable for the security model.

### Trade-off Accepted

**Performance sacrifice for security and embeddability.** If the IEPL execution engine needed to run complex algorithms or process large datasets, Boa would be the wrong choice. But in the COSMOS sandbox, JS code is a thin orchestration layer — its job is to call MCP tools in the right order, handle errors, and compose results. The actual heavy lifting is done by Rust-based agent tools. Boa's 10-100x slower execution speed compared to V8 is irrelevant when every tool call involves a network round-trip to Scepter that takes 50-500ms.
