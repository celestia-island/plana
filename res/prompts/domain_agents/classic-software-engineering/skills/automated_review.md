+++
name = "automated_review"
agent = "classic-software-engineering"
execution_mode = "read"
location = "cosmos"

[description]
en = "Automated code review pipeline: run static analysis, unwrap/expect audit, and clippy diagnostics, then produce a consolidated report."

[[related_tools]]
agent_name = "polemos"
tool_name = "host_command_exec"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_skills]]
agent_name = "skopeo"
tool_name = "smart_command_execute"
+++

# automated_review

Run a fully automated code review combining static analysis, error-handling audit, and dependency check. Produces a consolidated report with categorized findings.

## IMPORTANT: Host Path Convention

Use **host paths** (take the path from the environment section's `Workspace:` line, strip `local://`, and pass it as the `cwd` parameter of `host_command_exec`), NOT container paths (`/workspace`).

## SoP

### Step 1: CLIPPY PASS

Run `cargo clippy` with all warnings:

```text
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo clippy --message-format=json -- -W clippy::all 2>&1 | head -500', cwd: '<host-workspace>', timeout: 180 }); const out = r.data.stdout || r.data.stderr || ''; const findings = []; for (const line of out.split('\\n')) { try { const m = JSON.parse(line); if (m.reason === 'compiler-message' && m.message && m.message.level !== 'note') { const rendered = m.message.rendered || ''; if (rendered.includes('missing documentation')) continue; findings.push({ level: m.message.level, code: m.message.code?.code || '', msg: rendered.split('\\n')[0], file: m.message.spans?.[0]?.file_name, line: m.message.spans?.[0]?.line_start }); } } catch {} } write_to_var({ var_name: 'clippy', content: JSON.stringify(findings) }); console.log('Clippy:', findings.length, 'findings');" })
```

### Step 2: ERROR-HANDLING PASS

Audit `.unwrap()`, `.expect()`, `panic!()` in production code:

```text
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'rg \"\\\\.unwrap\\\\(\\\\)|\\\\.expect\\\\(|panic!|todo!\" --type rust -c 2>/dev/null | grep -v \"/tests/\" | sort -t: -k2 -rn | head -20', cwd: '<host-workspace>', timeout: 30 }); console.log(r.data.stdout || r.data.stderr);" })
```

### Step 3: FILE SIZE PASS

Find oversized files:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'find packages -name \"*.rs\" -exec wc -l {} + 2>/dev/null | sort -rn | head -15', cwd: '<host-workspace>', timeout: 30 }); console.log(r.data.stdout || r.data.stderr);" })
```

### Step 4: CONSOLIDATE & REPORT

Merge all findings, deduplicate, categorize by severity:

```json
write_to_var({ var_name: "rep", content: "CONSOLIDATED_SUMMARY" })
exec({ code: "import { report } from 'hubris'; report({ text: __vars['rep'] });" })
```

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Signal over noise**: Filter out documentation warnings, dependency noise, and generated code
- **Security findings always win**: If any security issue is found, it takes highest priority
- **Incremental scope**: For large repos, run against changed files first

> Return type: @system/return-type-convention
> IEPL-first: @system/iepl-first
