+++
name = "Smart Command Execution"
agent = "skopeo"

[[next_action]]
agent = "hubris"
name = "plan_execute"

[description]
en = "Intelligent command execution gateway: converts natural language to safe shell commands within containers, executes with security scanning, and intelligently compresses output. This is the sole command execution gateway — the upstream caller does NOT have direct access to exec."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "neikos"
tool_name = "exec_on_container"

[features]
execution_mode = "write"
location = "cosmos"
+++

Convert natural language commands into executable container instructions, execute them safely, and intelligently compress output to optimize context usage. This is the **sole command execution gateway** — the upstream caller does NOT have direct access to `exec`.

## SoP

1. **Parse intent** — Receive the natural language command request. Extract the target container, desired operation, and parameters. Use `llm_chat()` for complex intent resolution.
1. **Validate environment** — Confirm the target container exists and is running. Query container metadata (image, runtime, resource limits). If stopped, prompt to start it.
1. **Security scan** — Check the synthesized command against a blocked-pattern list (e.g., `rm -rf /`, `mkfs`). If a dangerous command is detected in safe mode, block execution and call `report_human()` for explicit approval with safe alternatives.
1. **Synthesize command** — Translate the natural language intent into a concrete shell command. If the translation is uncertain, present alternatives to the user for selection.
1. **Execute** — Run the command inside the target container with the configured shell, working directory, environment variables, and timeout. Capture stdout, stderr, and exit code.
1. **Handle errors** — If exit code is non-zero, classify the error type. Retry up to the configured count if the error is transient. For persistent failures, log diagnostics and report.
1. **Compress output** — If output exceeds `max_output_lines`, apply intelligent compression: retain error lines, summary lines, and key patterns; compress repetitive and verbose sections.
1. **Report** — Call `report()` with the execution summary: command, exit code, compressed output, timing, and any errors encountered.

> Return type and IEPL enforcement: @system/return-type-convention

## IEPL Preference Check

Before executing any shell command via `exec_on_container()`:

1. Can this be done with JavaScript string methods (match, replace, filter)?
1. If YES → use `exec()` with JavaScript code instead of `exec_on_container()`
1. If NO → proceed with `exec_on_container()` but document why IEPL is insufficient
