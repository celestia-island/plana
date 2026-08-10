+++
name = "Node Task Summary"
agent = "skopeo"

[description]
en = "This skill generates structured summary reports after node tasks are completed, recording key information, results, and lessons learned."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "skopeo"
tool_name = "goal_task_create"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[features]
execution_mode = "read"
location = "cosmos"
+++

Generate a structured summary report after a node task completes, capturing achievements, metrics, issues, and lessons learned.

## SoP

1. **Collect task data** — Gather execution logs, step records, timestamps, output artifacts, and performance metrics for the completed task.
1. **Aggregate achievements** — Identify the main outputs and deliverables. List each achievement with its completion status.
1. **Calculate metrics** — Compute performance indicators: total duration, resource utilization, error rate, test coverage, and any task-specific KPIs.
1. **Analyze issues** — Extract problems encountered during execution, their root causes, and the solutions applied. Assess impact severity.
1. **Extract lessons** — Use `llm_chat()` to synthesize reusable lessons learned and best practices from the execution history and issue analysis.
1. **Generate recommendations** — Propose specific follow-up actions with priority and owner. Create concrete next-step tasks via `goal_task_create()` when applicable.
1. **Compile report** — Assemble all sections into a structured report. Use the mandatory two-step reporting pattern: `write_to_var` for the multi-line body, then `exec` to call `report()` or `report_human()`. Pass the report content via `summary` (required) and `body` (detailed Markdown) parameters. See submit_report.md for the full convention.
1. **Archive knowledge** — Store successful patterns, failure patterns, and configuration insights for future task execution.

> Return type and IEPL enforcement: @system/return-type-convention
