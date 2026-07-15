+++
id = "cost-strategy-tokens"
title = "按令牌计费策略"
kind = "reference"
+++

# Cost Strategy — Per-Token Billing

The current model is billed **per token**, not per call. Cost scales with the
volume of data processed by expensive models, not with the number of requests.
This means frequent, precise reads are cheap and parallelism is free — the goal
is to minimize the total tokens consumed by the expensive model.

## Operating Principles

- **Use smart read/write freely.** Agentic, fine-grained reading is

cost-effective: each targeted read pulls only what you need, keeping the
expensive model's context lean.

- **Distill before forwarding.** Delegate filtering, search, and summarization to

cheaper models or sub-agents. Pass only the distilled result to the expensive
model so it processes fewer tokens.

- **Parallel reads are encouraged.** Multiple concurrent reads cost the same as

sequential ones (per-token), so fan out to gather context faster.

- **Prefer precision over brute-force.** Reading exactly the 30 relevant lines

costs fewer tokens than reading a 2000-line file. Use search/grep to narrow
before reading.

- **Keep the expensive model's context small.** Summarize, extract, and

compress. The fewer tokens the primary model processes, the lower the cost.

- **Re-read is acceptable when cheap.** If a small re-read avoids loading a

large stale buffer, it may save tokens overall. Optimize for token economy,
not call count.

## Trade-off Note

Because per-token billing rewards small, targeted operations, feel free to
iterate and explore — the cost is in *bytes*, not *requests*.
