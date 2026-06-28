+++
title = "ADR-001: Exec-Only Microkernel Tool Surface"
description = """Date: 2026-02"""
lang = "en"
category = "design"
subcategory = "core"
+++

# ADR-001: Exec-Only Microkernel Tool Surface

**Date**: 2026-02
**Status**: Accepted (amended 2026-06 — surface reduced from 5 to 3 primitives)

## Context

In a multi-agent LLM orchestration system, the model must decide which tools to call and how to compose them. The naive approach is to expose every MCP tool (118+ across 12 agents) directly to the LLM as separate function definitions in the prompt.

This creates several problems:

1. **Context window consumption**: 118+ tool definitions consume thousands of tokens, leaving less room for reasoning and conversation.
1. **Security surface area**: Every tool exposed to the LLM is a potential attack vector for prompt injection or jailbreaking.
1. **Permission enforcement fragmentation**: If tools are dispatched directly by LLM output, each tool must independently validate permissions — leading to inconsistent enforcement and gaps.
1. **Model confusion**: Research shows LLM performance degrades when presented with too many tool choices (the "tool overload" problem).

## Decision

We adopt an **exec-only microkernel** design. The LLM sees exactly **3 execution primitives** as its tool surface:

| Primitive | Purpose |
| --- | --- |
| `exec` | Execute TypeScript/JavaScript code via the IEPL pipeline |
| `write_to_var` | Write a string value to a named REPL variable |
| `write_to_var_json` | Write a JSON value to a named REPL variable |

> **Amendment (2026-06)**: The original design exposed 5 primitives, including `ref_add` and `ref_remove` for managing named reference variables. These two were **removed** from the LLM-visible surface because the reference-variable mechanism was unused by any skill SOP and added context overhead without value. The `agent_allowed_tools()` function in `packages/shared/domain_skills/src/tool_names.rs` now returns only the three primitives above. See commit history (`60d58f794`, `31cf9a00e`).

All 118+ agent MCP tools are invoked **indirectly** through ES module imports within the `exec` code. The LLM generates TypeScript code that imports and calls agent tools; this code is transpiled by SWC, validated by the AST security checker, and executed by the Boa JS engine inside the COSMOS sandbox.

## Consequences

### Positive

- **Minimal context overhead**: 3 tool definitions vs 118+. The LLM can focus on reasoning rather than tool selection mechanics.
- **Centralized security**: All tool calls pass through the `McpRouter` which enforces allowlists, dual-authorization, trust levels, and dynamic risk assessment in a single choke point.
- **Composable workflows**: The LLM can write arbitrarily complex tool compositions in TypeScript (loops, conditionals, error handling) rather than being limited to single tool calls.
- **Auditable execution**: Every `exec` call goes through AST validation that rejects dangerous constructs (`eval`, `require`, `process`, `Function`, `import()`, `globalThis` access).
- **Easier model migration**: New LLM providers only need to support function calling with 3 tools, not 118.

### Negative

- **Indirection overhead**: Tool calls go through TypeScript generation → SWC transpilation → AST validation → Boa execution → MCP router dispatch, adding latency to each call.
- **LLM code quality dependency**: The system's effectiveness depends on the LLM's ability to generate correct TypeScript code that properly imports and calls agent tools.
- **Debugging complexity**: When a tool call fails, the error chain spans TypeScript generation, transpilation, validation, JS execution, and MCP dispatch — making debugging harder than direct tool invocation.
- **Not all LLMs are equal**: Lower-capability models may struggle with code generation for complex multi-tool workflows compared to simple function calling.

### Risks Mitigated

- Prompt injection attacks that try to directly invoke dangerous tools
- Accidental tool misuse due to LLM confusion among too many choices
- Inconsistent permission enforcement across tools
