+++
name = "Instruction Failure Root Cause Backtrace"
agent = "epieikeia"

[description]
en = "`instruction_failure_backtrace` is the core diagnostic skill of the Epieikeia agent, specifically designed to analyze the root causes of instruction execution failures, providing detailed error diagnosis and remediation recommendations. This skill helps quickly locate and resolve faults in complex systems through intelligent backtrace analysis."

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
agent_name = "epieikeia"
tool_name = "deliver_message"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "consume_injected_prompts"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[features]
execution_mode = "read"
location = "cosmos"
+++

Diagnose instruction execution failures by capturing error context, tracing the causal chain back to the root cause, and producing prioritized remediation recommendations.

## SoP

1. **Capture Failure Context** — Record the failing instruction ID, error type, error message, timestamp, and full execution context. If context collection partially fails, preserve whatever is available and flag gaps.
1. **Classify Error** — Categorize the error as: `syntax`, `runtime`, `logic`, `timeout`, `resource`, or `dependency`.
1. **Build Causal Chain** — Trace backwards from the surface error through intermediate failures to the root cause. For each hop, record the failing component, the error propagated, and the link to the next hop. If the chain exceeds depth 50, truncate and mark as `max_depth_reached`.
1. **Correlate with History** — Match the current failure pattern against known historical patterns. Report the closest matches with similarity scores. If no match is found, flag for manual review.
1. **Determine Root Cause** — Synthesize the causal chain and historical matches into a single root cause statement. If uncertain, use `report_human()` to request guidance before proceeding.
1. **Generate Remediation Recommendations** — For each identified root cause, produce up to 5 prioritized remediation actions sorted by impact and difficulty. Include a brief rationale for each. If `auto_apply` is appropriate and low-risk, propose a dry-run first.
1. **Report and Archive** — Emit a structured report (see Output Format) via `report()`. For severity >= `critical` or unresolved failures, additionally use `report_human()`. Archive the analysis for future pattern matching.

> Return type and IEPL enforcement: @system/return-type-convention
