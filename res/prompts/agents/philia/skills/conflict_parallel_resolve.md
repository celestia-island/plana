+++
name = "Parallel Container Conflict Detection and Resolution"
agent = "philia"
[[next_action]]
agent = "skopeo"
name = "node_task_summary"

[description]
en = "This skill specializes in detecting and resolving conflict issues in parallel container operations, ensuring operation consistency and data integrity in multi-container environments."

[[related_tools]]
agent_name = "philia"
tool_name = "memory_query"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_store"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[features]
location = "cosmos"
execution_mode = "read"
+++

Detect and resolve conflicts in multi-container parallel operations to ensure data integrity and operational consistency.

## SoP

1. **Gather context** — Load current container states, shared resource manifests, and lock ownership data via `memory_query()`. Identify all active parallel operations and their access patterns (read/write).
1. **Analyze threats** — Classify detected conflicts by type (write-write, read-write, resource contention). Assess deadlock probability via dependency-cycle detection. Evaluate data corruption risk and cascading failure potential for each conflict.
1. **Decide strategy** — Select a resolution approach: lock-based coordination, optimistic concurrency with retry, last-writer-wins merge, or full serialization. Set lock granularity, timeout thresholds, and retry/backoff parameters.
1. **Execute resolution** — Acquire locks on contested resources, apply the chosen strategy, execute operations within transaction boundaries, synchronize state across containers, then release locks. Rebalance resource allocation post-resolution.
1. **Verify results** — Confirm all conflicts were resolved, validate data integrity across affected containers, ensure all locks were released, and check that no new conflicts were introduced.
1. **Report** — Compile resolution details via `report()`: conflict types, strategies applied, resolution timelines, and resource allocation changes. Escalate unresolved conflicts to `report_human()`.
1. **Capture knowledge** — Persist resolution patterns and effectiveness metrics to `memory_store()` for future conflict prediction and prevention.

> Return type and IEPL enforcement: @system/return-type-convention
