+++
id = "iepl-first"
title = "IEPL 优先执行规则"
kind = "reference"
+++

# IEPL-First Execution Rules

This document is injected into every skill prompt at assembly time. It defines the mandatory IEPL-first execution policy.

## Rule

When processing text, files, or data within the IEPL TypeScript runtime:

1. Use JavaScript/TypeScript string and array methods (`match`, `replace`, `filter`, `sort`, `split`, `join`, `map`, `reduce`) via `exec()` — NEVER use `sed`, `awk`, `grep`, `rg` or any shell text processing.
1. Only use `script_exec()` (imported from `skemma`) for genuine shell needs: Docker operations, network diagnostics, system information queries.
1. Never run `git checkout`, `git reset`, `git clean`, `git rebase` unless explicitly instructed by the human.

## Rationale

Shell commands produce unstructured text output that is difficult to validate and compose. JavaScript operations produce typed values that the IEPL runtime can verify against the skill's return type declaration.

## RAG Context Injection

Skill prompts now include pre-computed RAG context from two sources:

### Philia (Long-term Memory)

Injected into the **soul prompt section** at the end. Contains relevant memory nodes
retrieved via vector similarity + graph traversal.

```typescript
import { search as ragSearch } from 'rag/philia';
const result = JSON.parse(ragSearch('user preferences about X'));
// If empty, use import { memory_query } from 'philia' for live query
```

### Aporia (Knowledge / Workspace Index)

Injected into the **skill prompt section** at the end. Contains relevant documents
from the workspace RAG index.

```typescript
import { search as knowledgeSearch } from 'rag/aporia';
const result = JSON.parse(knowledgeSearch('workspace file content'));
// If empty, use import { workspace_search } from 'aporia' for live query
```

### Architecture

```text
Skill chain starts
  → rag_prefetch::prefetch_rag_context()
    → derive_ambient_query()
      ① LLM rewrite (ModelTier::Basic): skill + context → 2-3 search phrases
      ② Fallback: 30-word truncation
    → for each query:
      → aporia: embed → pgvector/cosine → merge into buffer
      → philia: embed → context_prepare() → merge into buffer
  → system prompt assembled with philia_section + aporia_section
  → IEPL modules read from SharedRagBuffer (same Arc as scepter writes)
```

The MCP tools remain available for follow-up queries when the pre-computed
context is insufficient. Buffer results are **merged** across queries (deduplicated),
not overwritten.
