+++
title = "Reflection Architecture: Continuous Self-Doubt in the Skill Chain"
description = """A three-tier reflection system that inserts structured self-doubt into the agent pipeline, turning isolated single-pass execution into a recursive quality loop."""
lang = "en"
category = "design"
subcategory = "core"
+++

# Reflection Architecture: Continuous Self-Doubt in the Skill Chain

> **Status**: Design specification. Implementation in progress.
> Written 2026-06-27.

## The Problem

The current skill chain pipeline executes in a **single forward pass**: a skill
produces output, the orchestrator checks structural correctness (did it call
`report()`? did it run `cargo check`?), then advances to the next skill. There is
no phase where the system asks:

1. *Was this step's reasoning sound?* (semantic reflection)
1. *Given what just happened, should we change direction?* (adaptive reflection)
1. *What should we remember for next time?* (lesson sedimentation)

The existing mechanisms — verify nudges, post-surgery rollback, YOLO daily audits —
are all **reactive and binary**: they detect technical failures, not reasoning
failures. A skill can produce syntactically correct, compiling code that solves
the wrong problem, and nothing in the current pipeline will catch it until a human
reviews the final output.

## Design Principles

1. **Reflection is a pipeline phase, not an afterthought hook.** It gets its own

slot between output validation and report dispatch — the same structural
weight as every other phase.

1. **Three tiers, three costs.** Not every step deserves deep philosophical

critique. The system must automatically select the appropriate reflection
depth based on what just happened.

1. **OreXis is the reflection agent.** Its existing design — the questioning

Titan, the one that pushes uncertainty to the operator — maps perfectly onto
the reflection role. No new agent needed.

1. **Lessons must flow forward.** A reflection that doesn't change future

behavior is just a diary. The lesson store must feed back into context
preparation so that the next chain benefits from the last chain's mistakes.

1. **Self-triggered reflection is a first-class tool.** Any agent should be able

to request a reflection cycle from within its IEPL script, not just wait for
the orchestrator to schedule one.

## Three-Tier Reflection System

```text
┌──────────────────────────────────────────────────────────────────┐
│                    SKILL CHAIN PIPELINE                          │
│                                                                  │
│  ┌──────┐   ┌──────┐   ┌──────┐   ┌──────┐   ┌──────────────┐  │
│  │ A:   │   │ D:   │   │ E:   │   │ F:   │   │ REFLECTION   │  │
│  │Guard │──▶│Build │──▶│Invoke│──▶│Valid │──▶│ (NEW)        │  │
│  │Check │   │Prompt│   │Skill │   │Report│   │              │  │
│  └──────┘   └──────┘   └──────┘   └──────┘   └──────┬───────┘  │
│                                                  │            │
│                                    ┌─────────────┼─────────┐ │
│                                    ▼             ▼         ▼ │
│                              ┌──────────┐ ┌─────────┐ ┌──────┐│
│                              │Tier 0:   │ │Tier 1:  │ │Tier 2││
│                              │Heuristic │ │Semantic │ │Deep  ││
│                              │(free)    │ │(1 LLM)  │ │(OreXis││
│                              └──────────┘ └─────────┘ │critiq││
│                                    │         │       │ue)   ││
│                                    ▼         ▼       └──┬───┘│
│                              ┌──────────────────────────────┐│
│                              │  ReflectionVerdict           ││
│                              │  Accept / Adjust / Backtrack ││
│                              │  / Abort                     ││
│                              └──────────────┬───────────────┘│
│                                             ▼                 │
│  ┌──────┐   ┌──────┐                   ┌──────────┐          │
│  │ G:   │   │ H:   │ ◀──────────────── │ Lesson   │          │
│  │Disp  │   │Stack │                   │ Store    │          │
│  │Report│   │Resolv│                   │ (write)  │          │
│  └──────┘   └──────┘                   └──────────┘          │
└──────────────────────────────────────────────────────────────────┘
```

### Tier 0: Heuristic Reflection (Zero-cost, always-on)

**What**: Rule-based checks on the skill output and execution trace.

**When**: Every skill, no exceptions.

**How**: Pure Rust logic, no LLM call. Expanded version of the existing
`validate_report_capture()` with additional heuristics:

- Output length sanity (too short / suspiciously long)
- Tool call pattern anomalies (called tools outside skill's expected domain)
- Execution time outliers (skill took 10x longer than its historical average)
- Error rate within tool calls (more than 50% of tool calls failed)
- Circular reference detection in output

**Cost**: Near zero (microseconds of CPU).

**Verdict**: Can emit `Accept` (pass) or `NeedsTier1` (escalate to semantic).

### Tier 1: Semantic Reflection (1 LLM call, at decision points)

**What**: LLM-based evaluation: "Does this output achieve the skill's stated
goal? Is the reasoning chain internally consistent?"

**When**: Triggered by:

- Tier 0 escalation (`NeedsTier1`)
- Skills explicitly marked as `requires_reflection` in their skill definition
- Skills at chain decision points (after `task_decompose`, `plan_execute`,

`workplan_generate`)

- First occurrence of a skill in a chain (novelty trigger)
- Self-triggered by any agent via `orexis::request_reflection` tool

**How**: OreXis agent invoked with a reflection-specific skill prompt. The prompt
includes:

- The skill's stated goal
- The skill's output
- The execution trace summary (tool calls made, their results)
- The chain context (what came before, what comes after)

**Cost**: 1 LLM call (~500-2000 output tokens).

**Verdict**: `Accept`, `Adjust` (with suggested modification), `Backtrack`
(return to previous skill and retry with different approach), or `NeedsTier2`
(escalate to deep critique).

### Tier 2: Deep Critique (OreXis philosophical review, at chain boundaries)

**What**: OreXis-driven examination of the entire chain: "Was the overall
approach correct? What assumptions were wrong? What should we learn?"

**When**: Triggered by:

- Tier 1 escalation (`NeedsTier2`)
- Chain completion (success or failure) — runs as a post-chain hook
- Operator-configured periodicity (e.g., every N chains)
- YOLO Strategic tier

**How**: OreXis agent with `deep_critique` skill. Reviews:

- The full chain trace (all skills, all outputs, all tool calls)
- The original goal vs. achieved outcome
- Historical lessons from the lesson store (for pattern matching)

**Cost**: 1-2 LLM calls (~1000-5000 output tokens).

**Verdict**: Produces a `DeepCritiqueReport` containing:

- Root-cause analysis (if chain failed)
- Assumption audit (which assumptions were valid, which weren't)
- Lesson candidates (to be written to lesson store)
- Confidence assessment

## Data Model

### ReflectionResult

```rust
pub struct ReflectionResult {
    pub tier: ReflectionTier,
    pub verdict: ReflectionVerdict,
    pub confidence: f32,
    pub reasoning: String,
    pub lessons: Vec<LessonCandidate>,
    pub suggested_adjustment: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub enum ReflectionTier {
    Heuristic,
    Semantic,
    Deep,
}

pub enum ReflectionVerdict {
    Accept,
    Adjust { modification: String },
    Backtrack { to_skill: String, reason: String },
    Abort { reason: String },
}
```

### Lesson Store

```rust
pub struct Lesson {
    pub id: Uuid,
    pub context_signature: String,      // semantic hash of the situation
    pub context_embedding: Vec<f32>,    // for similarity matching (pgvector)
    pub lesson_text: String,
    pub severity: LessonSeverity,
    pub source_chain_id: Uuid,
    pub source_skill: String,
    pub created_at: DateTime<Utc>,
    pub times_applied: u32,
    pub effectiveness_score: f32,       // updated when lesson is applied
    pub deprecated: bool,
}

pub enum LessonSeverity {
    Info,
    Warning,
    Critical,
}
```

### ReflectionTrigger

```rust
pub enum ReflectionTrigger {
    Always,                              // Tier 0 for every skill
    SkillFlagged(String),                // skill has requires_reflection
    DecisionPoint,                       // task_decompose, plan_execute, etc.
    NovelSkill,                          // first occurrence in chain
    Escalation(ReflectionTier),          // lower tier escalated
    SelfRequested { by_agent: String },  // orexis::request_reflection
    PostChain,                           // after chain completes
    Periodic { interval_secs: u64 },     // YOLO-style periodic
}
```

## Integration Points

### 1. Pipeline Insertion (pipeline.rs)

Between Phase F (`post_execution_cleanup`) and Phase G
(`dispatch_report_and_check_termination`), insert:

```rust
// NEW: Reflection phase
let reflection_result = Self::reflect_on_output(
    st, &s, &setup, &invoke_result, &cleanup,
).await;

match reflection_result.verdict {
    ReflectionVerdict::Accept => { /* proceed to Phase G */ },
    ReflectionVerdict::Adjust { modification } => {
        // Inject modification into next skill's context
        s.pending_adjustment = Some(modification);
    },
    ReflectionVerdict::Backtrack { to_skill, reason } => {
        // Roll back chain state to the specified skill
        s.current_skill = to_skill;
        s.executed_skills.remove(&to_skill);
        // Continue loop from the earlier skill
    },
    ReflectionVerdict::Abort { reason } => {
        break 'chain false;
    },
}
```

### 2. Context Preparation (lesson injection)

In Phase D (`build_prompts`), before assembling the system prompt, query the
lesson store for contextually relevant lessons:

```rust
let relevant_lessons = lesson_store
    .find_similar(&s.current_skill_context, TOP_K_LESSONS)
    .await;

let lesson_section = format_lessons_for_prompt(&relevant_lessons);
// Inject into system prompt after skill description
```

### 3. Self-Triggered Reflection (IEPL tool)

Expose `orexis::request_reflection` to the IEPL namespace. Any agent can call
this within its TypeScript code:

```typescript
import { request_reflection } from 'orexis';

const review = await request_reflection({
  reason: "I'm about to execute a potentially destructive operation",
  context: { operation: "file_delete", path: "/etc/..." },
  urgency: "high",
});
```

This spawns an OreXis reflection cycle synchronously, blocking the calling
agent until the reflection completes.

### 4. Post-Chain Deep Critique Hook

Register a new pipeline hook in the `surgery_hooks` namespace:

```text
pipeline.reflection.post_chain  (priority 30, after PostSurgeryRollback)
```

This hook runs Tier 2 deep critique after a successful chain, producing
lessons that benefit future chains.

## Lesson Lifecycle

```text
┌─────────────┐     ┌──────────────┐     ┌───────────────┐
│  Created    │────▶│  Applied     │────▶│  Evaluated    │
│  (Tier 2)   │     │  (injected   │     │  (did it help?│
│             │     │   into next  │     │   track score)│
│             │     │   chain)     │     │               │
└─────────────┘     └──────────────┘     └───────┬───────┘
                                                 │
                                    ┌────────────┼────────────┐
                                    ▼            ▼            ▼
                              ┌──────────┐ ┌──────────┐ ┌──────────┐
                              │Reinforced│ │Adjusted  │ │Deprecated│
                              │(score ↑) │ │(text     │ │(score ↓, │
                              │          │ │ updated) │ │ removed) │
                              └──────────┘ └──────────┘ └──────────┘
```

Lessons are living artifacts:

- **Reinforced** when applying them correlates with successful outcomes
- **Adjusted** when the lesson text needs refinement based on new evidence
- **Deprecated** when they consistently fail to help or become irrelevant

## Cost Management

| Tier | Token Cost | Frequency | Daily Token Budget |
| --- | --- | --- | --- |
| Heuristic | 0 | Every skill | 0 |
| Semantic | ~2K tokens | Decision points (~30% of skills) | ~20K tokens/chain |
| Deep | ~5K tokens | Per chain + escalations | ~5K-10K tokens/chain |

Total overhead: approximately 10-15% additional token consumption per chain,
trading tokens for quality. Configurable via environment variables.

## Configuration

```env
# Reflection system configuration
REFLECTION_ENABLED=true
REFLECTION_TIER0_ENABLED=true                    # always-on heuristics
REFLECTION_TIER1_ENABLED=true                    # semantic reflection
REFLECTION_TIER2_ENABLED=true                    # deep critique
REFLECTION_TIER1_SKILL_THRESHOLD=0.3             # % of skills that get Tier 1
REFLECTION_POST_CHAIN_ENABLED=true               # deep critique after chain
REFLECTION_LESSON_TOP_K=5                        # max lessons injected per skill
REFLECTION_LESSON_MIN_EFFECTIVENESS=0.3          # don't inject useless lessons
REFLECTION_BACKTRACK_MAX=2                       # max backtracks per chain
```

## Relationship to Existing Systems

| Existing System | Relationship |
| --- | --- |
| `validate_report_capture()` | Subsumed by Tier 0 as one heuristic among many |
| Verify nudge | Becomes a Tier 0 heuristic that can escalate to Tier 1 |
| `PostSurgeryRollback` | Remains as-is; reflection runs before it, not replacing it |
| YOLO Daily audits | Complemented by reflection; audits check compliance, reflection checks reasoning |
| `RetroReview` | Subsumed by Tier 2 deep critique for goal-level review |
| `quality_score` / `lesson` fields | Connected to lesson store; finally have a reader |
| `context_preparation` | Extended to inject relevant lessons |
| `agent-consensus` | Consensus validates facts; reflection validates reasoning. Complementary |
| `memory-and-self.md` | Lesson store is the operational implementation of "memory sedimentation for self" |

## Naming

The reflection system's internal components follow the existing Greek
philosophical naming convention:

- **OreXis** (ὄρεξις, "desire/yearning") — already the questioning agent.

Its desire is to know whether things are *right*, not just *done*.

- **Lesson** — consistent with the existing `quality_score` / `lesson` field

names in `TaskState`.

- **`ReflectionResult`** — functional naming, matching `ReportCaptureDecision`

and other pipeline types.
