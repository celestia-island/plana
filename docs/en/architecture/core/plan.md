+++
title = "Industrial Corridor Integration Plan"
description = """> Goal: The system must demonstrate autonomous self-interfacing with a completely"""
lang = "en"
category = "architecture"
subcategory = "core"
+++

# Industrial Corridor Autonomous Integration Plan

> **Goal**: The system must demonstrate **autonomous self-interfacing** with a completely
> unknown industrial demonstration corridor — discovering hardware, inferring data models,
> generating monitoring configuration, and closing the alarm→response loop — without
> manual per-device engineering.
> **Government hard deadline**: this capability is bound to a government project milestone.

---

## Remaining Work

The full discovery → inference → monitoring → alarm → **write-approval** chain is
shipped (Phases A.1–A.3, B, C, D.1, **D.2 ✓**). The only remaining work is the
**end-to-end dogfood validation (Phase E)** — operational, not code.

### D.2 — Write-approval round-trip (human-in-the-loop) ✓

```text
Agent decides write is needed
  → verify_write_safety → Denied
    → orexis.request_write_approval → WriteApprovalRequest broadcast
      → shittim-chest shows approval dialog (industrial.approveWrite)
        → [approved] → temporary whitelist entry → execute + read-back verify
        → [denied]   → agent receives denial, adjusts plan
```

**Implemented:**

| # | Task | File | Status |
| --- | --- | --- | --- |
| A.2.4.1 | `orexis.request_write_approval` MCP tool — builds `WriteApprovalRequest`, broadcasts `TuiMessage::IndustrialWriteApprovalPush`, suspends (oneshot + timeout) until operator responds | `packages/agents/orexis/src/mcp/tools/industrial_write_tools.rs` | ✓ |
| A.2.4.2 | `industrial.approveWrite` WS handler — resolves pending request via shared `WriteApprovalRegistry`; on approval adds a temporary whitelist entry so the subsequent write passes `verify_write_safety` | `packages/scepter/src/tui_connection/mod.rs` | ✓ |

The producer/resolver are decoupled via a process-wide shared
`WriteApprovalRegistry` (`_shared_security_policy::write_approval_registry`),
injected into orexis at startup and used by scepter when the operator responds.

---

## Phase E: End-to-End Dogfood

Operational validation, not pure code. Requires running hardware simulators.

### E.1 — Test environment

| # | Component | Setup |
| --- | --- | --- |
| E.1.1 | S7comm simulator | Run `snap7-server` crate as virtual S7-1500. Pre-load DB1 with: REAL temp at offset 0, REAL pressure at offset 4, INT flow at offset 8, BOOL valve at offset 10, plus 50 bytes of random data |
| E.1.2 | Modbus simulator | Run aoba slave mode on virtual serial port (`socat pty pty`). Pre-load station 5 with known register values |
| E.1.3 | Entelecheia + evernight | Standard docker-compose startup. evernight `sensor-poll` ready with `--manifest` flag |

### E.2 — Dogfood scenarios

| # | Scenario | Steps | Pass criteria |
| --- | --- | --- | --- |
| E.2.1 | **Unknown S7comm corridor** | (1) Give system target `192.168.1.10:102`. (2) `industrial_discover` skill chain runs autonomously. (3) System discovers S7comm protocol, DB1, infers field semantics, generates manifest. (4) Operator reviews manifest in TUI. (5) Approve → evernight starts polling. (6) Inject alarm value → Hubris alarm_response triggers → corrective action proposed. | Manifest generated with ≥ 3 correctly inferred fields. Alarm triggers `alarm_response → task_decompose → plan_execute` chain. |
| E.2.2 | **Unknown Modbus corridor** | Same flow but with Modbus RTU on virtual serial port. Different station layout. | Same criteria. |
| E.2.3 | **Mixed-protocol discovery** | Run both simulators simultaneously. System discovers both, generates combined manifest. | Both stations appear in manifest with correct protocols. |
| E.2.4 | **Write-approval flow** | Agent proposes closing a valve (write to discovered BOOL field). `verify_write_safety` blocks (not whitelisted). WriteApprovalRequest sent to operator. Operator approves. Write executes with read-back verification. | Full round-trip: propose → block → request → approve → execute → verify. **(D.2 now shipped — ready to dogfood.)** |

### E.3 — Demo recording

| # | Task | Notes |
| --- | --- | --- |
| E.3.1 | Record full discovery→monitoring→alarm→response cycle as screen capture | Demonstrate autonomous adaptation to unknown hardware |
| E.3.2 | Generate discovery report artifact (auto-generated manifest TOML + inferred field table) | Tangible deliverable for government milestone review |

---

## Dependency on Sibling Projects (remaining)

| Sibling | What we need from them | When | Status |
| --- | --- | --- | --- |
| **arona** | WS broadcast path for `WriteApprovalRequest` (A.2.4) | ~~blocks A.2.4 / D.2~~ done — rides `TuiMessage::IndustrialWriteApprovalPush` (re-exported from arona types) | ✓ |
| **shittim-chest** | Operator approval dialog (`industrial.approveWrite` consumer) + discovery progress rendering | blocks E.2.4 dogfood (the WS handler in scepter is ready; shittim-chest needs to render the dialog and POST the response) | sibling PLAN |

---

## Explicitly Out of Scope (2-week sprint)

- OPC UA client/server (Rust ecosystem not ready)
- EtherNet/IP / CIP (Rockwell)
- EtherCAT (Beckhoff)
- CAN bus
- Frontend test coverage (shittim-chest gets guidance plan only, no test-writing)
- CLI feature parity with TUI

---

# Technical Roadmap — Architecture Deepening

> **Date**: 2026-06-26
> **Context**: After cleaning the repo of 700+ stale docs/files and consolidating all prompts into `res/prompts/`, we audited the remaining design docs against actual source code to identify which aspirational designs are worth implementing.

---

## 1. Sub-Badge Addressing + Parallel Skill Execution

**Verdict**: Worth implementing. Infrastructure ~80% built, missing only the final 20%.

**Current state**:

- `BadgeRegistry` (`packages/scepter/src/state_machine/badge_registry.rs:92-120`) already supports parent-child `link_sessions()`.
- `#001.005` sub-badge syntax parsing exists in `find_by_container_id_or_sub()` but strips the sub-number instead of resolving to a distinct child container.
- `SnowflakeContainer.parent_id` and `branch_level` fields exist but are metadata-only — never used for routing.
- Edge node priority queuing (`edge_node_registry.rs:73-126`) is ready for fine-grained resource locking.
- Skill chain is strictly **serial** — `pipeline.rs:68-226` loops one skill at a time. Coordinator skills with independent `next_targets` run serially when they could run in parallel.

**What's missing**:

1. ✅ Make `find_by_container_id_or_sub()` resolve `#001.005` → the deepest

active forked child of the parent container, falling back to the parent when
no fork exists (backward compatible).

1. ✅ Add child/descendant lookup to `SnowflakeManager`: `children_of`,

`children_of_badge`, `most_recent_child_of`, `deepest_descendant`
(`parent_id` → reverse index).

1. ✅ `FuturesUnordered`-based parallel execution of `next_targets`:

`dispatch_parallel_targets` fans a coordinator's independent **leaf**
targets out concurrently via `parallel_dispatch::fan_out` (bounded by a
`Semaphore`). The two global-singleton blockers in the serial
`invoke_skill_with_retries` path are handled as follows:

   - **Shared local cosmos namespace** → each target is forked into its **own

cosmos container** in Phase 1 (`fork_container_for_skill` +
`assign_container_id` + `register_container_badge_in_registry`), so
`dump/restore_cosmos_namespace` is a no-op per branch and concurrent exec
is isolated. `MAX_BRANCH_DEPTH` (item 4) bounds the fork chain.

   - **`active_streaming_skill` UI race** → tolerated (last-writer-wins on an

`Option`; reset to `None` after each branch).

   - **`&mut SkillChainInput` threading** → `BranchOwner` mirrors the mutable

portions per branch; `as_input` borrows them back into a short-lived
`SkillChainInput` so the unchanged pipeline helpers are reused.
Phase 1 (fork + prepare + build prompts + tool whitelist) is **serialized**
to avoid `rag_buffer` races; only Phase 2 (the latency-dominant LLM
invocations) runs in parallel; Phase 3 cleans up and merges reports
(`merge_branch_reports`) into the parent context. Gated behind
`SKILL_CHAIN_PARALLEL_TARGETS` (default **off**) +
`parallel_targets_eligible` (containerized + all-leaf targets). The serial
stack-unwind in `route_to_next_skill` remains the default.

1. ✅ Enforce `MAX_BRANCH_DEPTH` (`COSMOS_MAX_BRANCH_DEPTH`, default 4) in both

fork paths; children now register at `source.branch_level + 1` instead of a
hardcoded `1`.

**Expected impact**: Parallel file writes, parallel analyses from coordinator skills like `industrial_discover` would reduce end-to-end latency significantly.

---

## 2. Memory Sedimentation Pipeline

**Verdict**: Quality multiplier, not critical. Reserved for long-term roadmap.

**Current state**:

- `PhiliaMemoryService` is a flat "store → embed → retrieve" graph with no metabolism.
- `memory_consolidate` is trivial — just creates an episode node, no abstraction/summarization.
- No memory decay, aging, staleness scoring, or quality gradient across nodes.
- All nodes are undifferentiated `MemoryNode` — no episodic/procedural/atomic separation.
- In-memory vector search is O(n) brute-force (won't scale long-term).
- `KnowledgeStore` (separate system) has lifecycle stages (Created→Vectorized→Searchable→Consolidated→Deprecated) and consensus validation — this closest existing analogue to sedimentation.

**Why it's not urgent**:

- RAG context injection (`RagContextBuffer` → LLM query rewrite → `bundle_search`) provides sufficient context for current tool-calling agents.
- pgvector HNSW index handles production-scale retrieval.
- The system works as "store and retrieve" — sedimentation would make it "metabolize," but this is incremental quality, not functional gap.

**Future work** (no timeline):

- Auto-consolidation: periodic LM-driven summarization of related nodes into higher-level "episodes."
- Quality gradient: access counts, temporal decay, confidence scoring.
- Three-channel prototype (episodic/procedural/atomic) with differentiated retrieval strategies.

---

## 3. Inter-Agent Negotiation

**Verdict**: Low priority. Primitives exist as low-level building blocks; no immediate use case.

**Current state**:

- `deliver_message(message_type="Question")` exists (`epieikeia/src/mcp/tools/deliver_message.rs:63`) — can push questions to another agent's mailbox.
- `inject_user_prompt` / `consume_injected_prompts` exist but are **poll-based** — no pipeline integration. Agents must explicitly call `consume_injected_prompts` to check mail.
- `Haplotes` has `AskAgent` / `ReplyAgent` / `Escalated` conversation routing types — but all are no-op ACKs with zero business logic.
- `NEGOTIATION_ROUND_TIMEOUT_SECS` / `NEGOTIATION_TOTAL_TIMEOUT_SECS` env vars are defined in `RuntimeTuningConfig` but **never consumed** anywhere — dead code.

**Why it's low priority**:

- Current sequential skill-chain dispatch + context-as-string passing handles all current use cases.
- Merge conflicts are handled by single-skill dispatch (`resolve_merge_conflict`), which is sufficient.
- The negotiation loop (intercept skill chain → ask agent → wait for response → incorporate) would be complex to build and test. No production use case demands it yet.

**When to revisit**: If agents ever need to dynamically negotiate mid-chain decisions (not just dispatch-and-wait), the primitives are 40% built. The gap is the pipeline integration loop.

---

## Summary

| Feature | Infra built | Priority | Next step |
| --- | --- | --- | --- |
| Sub-badge + parallel exec | 100% | **High** | ✅ Done — sub-badge→child, children index, branch-depth & in-loop parallel dispatch all shipped (parallel default-off) |
| Memory sedimentation | 20% | **Long-term** | No immediate action; revisit after parallel exec |
| Inter-agent negotiation | 40% | **Low** | Wait for concrete use case; primitives are ready |
