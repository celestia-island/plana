+++
name = "static_analyze"
agent = "classic-software-engineering"
execution_mode = "read"
location = "cosmos"

[description]
en = "Run language-specific static analysis (cargo clippy, eslint, pylint, go vet) via host_command_exec. Parses structured output and returns categorized findings with file/line/severity."

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

# static_analyze

Run language-specific static analysis tools on the workspace and return categorized, actionable findings.

## IMPORTANT: Host Path Convention

This skill runs on the **host** via `host_command_exec`. Use **host paths** (take the path from the environment section's `Workspace:` line, strip `local://`, and pass it as the `cwd` parameter of `host_command_exec`), NOT container paths (`/workspace`).

## Analysis Commands by Language

| Language | Detection File | Scan Command | Output Format |
| --- | --- | --- | --- |
| Rust | `Cargo.toml` | `cargo clippy --message-format=json -- -W clippy::all 2>&1` | JSON lines |
| TypeScript/JS | `package.json` | `npx eslint --format json --no-color . 2>&1` | JSON array |
| Python | `pyproject.toml` / `setup.py` | `python -m pylint --output-format=json . 2>&1` | JSON array |
| Go | `go.mod` | `go vet ./... 2>&1` | text |
| Java | `pom.xml` | `mvn compile 2>&1` | text |

## CRITICAL: Single Exec Rule

You MUST run the scan command in a **single** `host_command_exec` call. Do NOT split detection and scanning into separate calls.

## SoP

### Step 1: DETECT & SCAN (mandatory)

Run the appropriate scan command. Use the workspace path from the environment:

```typescript
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo clippy --message-format=json -- -W clippy::all 2>&1 | head -200', cwd: '<host-workspace>', timeout: 180 }); const out = r.data.stdout || r.data.stderr || JSON.stringify(r.data); const lines = out.split('\\n'); const findings = []; for (const line of lines) { try { const msg = JSON.parse(line); if (msg.reason === 'compiler-message' && msg.message) { findings.push({ level: msg.message.level, code: msg.message.code?.code || 'unknown', message: msg.message.rendered?.trim(), spans: (msg.message.spans || []).map(s => ({ file: s.file_name, line_start: s.line_start, line_end: s.line_end })) }); } } catch {} } write_to_var({ var_name: 'dag', content: JSON.stringify(findings) }); console.log('Findings:', findings.length, 'Errors:', findings.filter(f => f.level === 'error').length, 'Warnings:', findings.filter(f => f.level === 'warning').length);" })
```

**Language-specific templates (copy-paste the one you need):**

#### Rust — cargo clippy (structured JSON)

```typescript
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo clippy --message-format=json -- -W clippy::all 2>&1 | head -500', cwd: '<host-workspace>', timeout: 180 }); const out = r.data.stdout || r.data.stderr || ''; const findings = []; for (const line of out.split('\\n')) { try { const m = JSON.parse(line); if (m.reason === 'compiler-message' && m.message && m.message.level !== 'note') { findings.push({ level: m.message.level, code: m.message.code?.code || '', msg: m.message.rendered?.split('\\n')[0], file: m.message.spans?.[0]?.file_name, line: m.message.spans?.[0]?.line_start }); } } catch {} } write_to_var({ var_name: 'dag', content: JSON.stringify(findings) }); console.log(JSON.stringify({ total: findings.length, errors: findings.filter(f=>f.level==='error').length, warnings: findings.filter(f=>f.level==='warning').length }, null, 1));" })
```

#### Rust — cargo check (compilation errors only)

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cargo check 2>&1 | tail -50', cwd: '<host-workspace>', timeout: 180 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

#### TypeScript/JavaScript — eslint

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'npx eslint --format json --no-color . 2>&1 | head -200', cwd: '<host-workspace>', timeout: 120 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

#### Python — pylint

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'python -m pylint --output-format=json . 2>&1 | head -200', cwd: '<host-workspace>', timeout: 120 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

#### Go — go vet

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'go vet ./... 2>&1', cwd: '<host-workspace>', timeout: 120 }); console.log(r.data.stdout || r.data.stderr || JSON.stringify(r.data));" })
```

### Step 2: FILTER (mandatory)

Filter out noise. Exclude:

- `missing documentation` warnings (not actionable)
- `generated N warnings` summary lines
- Warnings from dependency crates (paths containing `.cargo/`, `node_modules/`)
- Patch/version warnings

Only keep actionable findings: unused imports, unused variables, dead code, type errors, clippy style issues, real compilation errors.

### Step 3: REPORT

```json
write_to_var({ var_name: "rep", content: "FINDINGS_SUMMARY" })
exec({ code: "import { report } from 'hubris'; report({ text: __vars['rep'] });" })
```

## Decision Philosophy

@system/decision-philosophy

**Skill-specific principles:**

- **Signal over noise**: Filter aggressively. Only report findings the developer can act on.
- **One scan per call**: Never run `cargo clippy` then `cargo check` separately — they overlap.
- **JSON parsing first**: Always try structured parsing. Fall back to text parsing only if JSON fails.
- **Respect timeout**: Set `timeout: 180` for large workspaces. If it times out, narrow scope to specific packages.

> Return type: @system/return-type-convention
> IEPL-first: @system/iepl-first
