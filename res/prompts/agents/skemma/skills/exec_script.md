+++
name = "exec_script"
agent = "skemma"

[[next_action]]
agent = "hubris"
name = "plan_execute"

[description]
en = "Intelligent script execution gateway with language detection, sandboxing, and structured output"
zh-Hans = "智能脚本执行网关：语言检测、沙箱隔离、结构化输出"

[[related_tools]]
agent_name = "skemma"
tool_name = "script_exec"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
location = "cosmos"
execution_mode = "write"
+++

# exec_script

Execute scripts safely based on upstream context. This skill is the **gateway** for script execution — the upstream caller does NOT have direct access to `script_exec()`.

## SoP

1. **Parse request** — The upstream context describes what computation or script is needed. Extract the script content, language hint, and execution requirements.
1. **Language detection** — Determine the script language from content heuristics (shebang, syntax patterns, file extension). Default to bash if ambiguous.
1. **Security scan** — Reject dangerous patterns: `rm -rf /`, `mkfs`, `dd if=/dev/zero`, network exfiltration, privilege escalation. If uncertain, use `report_human()`.
1. **Execute** — Use `script_exec()` with appropriate parameters:

   - Set `timeout` based on complexity (default 120s, max 300s)
   - Set `memory_limit` based on expected usage (default 512MB, max 2048MB)
   - Capture stdout, stderr, and exit code

1. **Interpret results** — Parse output for the upstream caller. If the script failed, analyze stderr and provide actionable diagnostics.
1. **Report** — Call `report()` with structured findings.

> Return type and IEPL enforcement: @system/return-type-convention

## Edge Cases

- **Timeout**: Report which step was slow, suggest splitting the script
- **OOM**: Report memory usage, suggest reducing data size or streaming
- **Syntax error**: Report the exact line and column, suggest a fix
- **Permission denied**: Report the error, suggest alternatives or `report_human()`
- **No script provided**: Ask for clarification via `report_human()`
