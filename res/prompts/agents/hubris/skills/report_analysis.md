+++
name = "report_analysis"
agent = "hubris"

[description]
en = "Read recent YOLO reports, extract actionable issues, register them as tech_debt TODOs for automated repayment."
zhs = "读取最近的 YOLO 报告，提取可操作问题，注册为 tech_debt TODO 以供自动偿还。"

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
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "polemos"
tool_name = "host_command_exec"

[features]
execution_mode = "read"
location = "cosmos"
+++

# report_analysis

Read recent reports from `~/.config/entelecheia/reports/`, extract actionable issues, and register them as `tech_debt` TODOs. This skill closes the self-iteration loop: reports identify problems → this skill registers them → `yolo_cycle_report` picks them up → `plan_execute` fixes them.

## SoP

1. **Read recent reports**: Use `host_command_exec` to list and read the most recent reports (last 24 hours):

   ```json
   exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'ls -t ~/.config/entelecheia/reports/*.md | head -10', timeout: 5 }); console.log(r.data?.stdout || r.data?.stderr);" })
   ```

Then read each file's content with `host_command_exec` + `cat`.

1. **Check existing TODOs**: Query existing `tech_debt` items to avoid duplicates:

   ```typescript
   exec({ code: "import { list_todo } from 'hubris'; const todos = await list_todo({ view: 'flat', tag: 'tech_debt' }); console.log(JSON.stringify(todos.data?.items?.map(t => t.title) || []));" })
   ```

1. **Analyze with LLM**: Send the combined report summaries to `llm_chat` with a prompt:

   - "Given these system reports, extract actionable issues that can be fixed in code. For each issue: title, severity (P0-P3), category (bug/performance/security/docs), specific file:line if mentioned, and a one-line fix suggestion. Ignore issues that are 'cold start' or 'empty database' — focus on code-level problems."
   - Parse the LLM response as structured data.

1. **Register new TODOs**: For each issue NOT already in the existing `tech_debt` list:

   ```json
   exec({ code: "import { create_todo } from 'hubris'; await create_todo({ title: 'ISSUE_TITLE', description: 'FILE:LINE — FIX_SUGGESTION', metadata: { tags: ['tech_debt', 'priority:SEVERITY', 'source:report_analysis'], category: 'CATEGORY' } });" })
   ```

1. **Report**: Summarize what was found and registered:

   - Reports scanned: N
   - Issues extracted: N
   - New TODOs registered: N (skipped N duplicates)
   - Top 3 highest-priority items

## Rules

- **Dedup**: Never register a TODO that duplicates an existing one (check titles).
- **Actionable only**: Ignore "cold start", "empty database", "0 results" findings — these are environmental, not code bugs.
- **Severity**: Reserve P0 for crashes/data-loss. P1 for broken features. P2 for degraded performance. P3 for cleanup/docs.
- **Source tracking**: Every TODO must have `source:report_analysis` in metadata so we can trace where it came from.
