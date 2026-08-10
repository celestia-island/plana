+++
name = "Degradation Detection"
agent = "epieikeia"

[description]
en = "`degradation_check` is a critical skill of the Epieikeia agent, used to detect system degradation status in real-time, automatically triggering degradation strategies to ensure system maintains core service availability when some functions fail. This skill improves system fault tolerance and business continuity through intelligent monitoring and automated response."

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
tool_name = "fork_container_on_next_action"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "list_file_observers"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_skills]]
agent_name = "epieikeia"
tool_name = "instruction_failure_backtrace"

[features]
execution_mode = "read"
location = "cosmos"
+++

Detect system degradation in real-time by collecting metrics, evaluating thresholds, applying mitigation strategies, and producing a structured report.

## SoP

1. **Collect Metrics** — Gather current system performance data: CPU usage, memory usage, disk usage, error rate, response time, and external dependency health. Use cached last-known values if collection fails and flag staleness.
1. **Evaluate Thresholds** — Compare collected metrics against configured thresholds (CPU 80%, memory 85%, disk 90%, error rate 5%, response time 5000ms). If multiple metrics breach, classify at the highest applicable degradation level.
1. **Classify Degradation Level** — Assign one of: `minimal`, `partial`, `severe`, `critical`. When trend data is ambiguous, default to the more conservative (higher) level. Treat timed-out dependency checks as unavailable.
1. **Capture Pre-degradation Snapshot** — Save the current system state before applying any changes. If snapshot capture fails, proceed but log the gap.
1. **Select and Execute Strategy** — Choose the matching degradation strategy for the classified level. Identify non-core services to shut down, apply rate limiting and circuit breaking, and extend cache TTLs. If no matching strategy exists, fall back to the next higher level's strategy. Retry failed commands once; escalate to manual intervention on second failure.
1. **Verify Results** — Confirm non-core services are stopped, rate limits are active, core service response times have improved, and error rates are trending downward. If core services do not improve, escalate to `critical`.
1. **Report and Capture Knowledge** — Generate a degradation report (see Output Format). Record the event, root cause, strategy effectiveness, and update threshold tuning recommendations. Use `report()` for automated delivery and `report_human()` for severity >= `severe`.

> Return type and IEPL enforcement: @system/return-type-convention
