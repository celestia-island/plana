+++
name = "YOLO Cycle Meta-Report"
agent = "hubris"

[description]
en = "Analyze YOLO auto-cruise cycle results and produce a meta-report summarizing system health, skill execution outcomes, token usage trends, and anomalies across all YOLO tiers."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "list_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "create_todo"

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_close"

[features]
execution_mode = "read"
location = "cosmos"
+++

## YOLO Cycle Meta-Report

Produces a summary of YOLO auto-cruise activity across all tiers (Periodic + Daily). This is a meta-analysis skill — it examines what YOLO has been doing and reports trends, not individual results.

## SoP

1. **Gather cycle history**: Review the YOLO daemon's recent execution log. For each tier:

   - Periodic (1h): How many cycles completed in the last 24 hours?
   - Daily (6h): How many cycles completed in the last 24 hours?
   - For each completed cycle: which skills ran, did they succeed or fail?

1. **Token usage analysis**: Summarize cumulative token consumption by YOLO:

   - Total tokens used by Periodic tier skills
   - Total tokens used by Daily tier skills
   - Compare to previous cycle (is usage trending up/down/stable?)
   - Flag any single skill that consumed >50% of the cycle's total tokens

1. **Skill success rate**: For each YOLO skill, compute:

   - Success count / total invocations
   - Average execution time
   - Any skills that failed consecutively (2+ failures = needs attention)

1. **Anomaly detection**: Flag:

   - Skills that consistently time out (may need SoP simplification)
   - Skills that produce empty reports (may need tool access fixes)
   - Any tier that was disabled or skipped during the period
   - Unusual token spikes (>2x average for a single invocation)

1. **System health indicators**: Derive from skill results:

   - Memory graph: growing/stable/shrinking (from `memory_consolidate` output)
   - RAG index: healthy/degraded (from `knowledge_base_health` output)
   - Security: issues found/issues resolved (from `security_audit` output)
   - Code quality: violations found/fixed (from `code_review_standard` output)

1. **Tech debt scan**: Query TODO items with `tech_debt` tag:

   ```typescript
   exec({ code: "import { list_todo } from 'hubris'; const todos = await list_todo({ view: 'tree' }); const debts = todos.data?.items?.filter(t => t.metadata?.tags?.includes('tech_debt')) || []; console.log(JSON.stringify(debts, null, 2));" })
   ```

   - If `tech_debt` items exist and no critical YOLO failures are blocking: **dispatch the highest-priority `tech_debt` item for repayment** by reporting it as a recommendation.
   - If a `tech_debt` item is stale (>7 days old) or blocked: flag it for human review.

1. **Generate report**: Use `report()` with type `system` containing:

   - Executive summary (1-2 sentences)
   - Per-tier execution table (skill, status, tokens, duration)
   - **Tech debt inventory** (count, highest priority item, recommended action)
   - Anomaly list (if any)
   - Trend indicators (token usage, success rate)
   - Recommendations (if any skills need adjustment)

## Decision Philosophy

- **Observation only**: This skill never modifies state. It reads and reports.
- **Actionable over verbose**: If everything is healthy, the report should be short. Only elaborate on anomalies.
- **Trend-aware**: A single failure is noise. Consecutive failures are signal.
