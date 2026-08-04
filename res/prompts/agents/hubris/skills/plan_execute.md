+++
name = "plan_execute"
agent = "hubris"

[features]
execution_mode = "write"
must_touch_next_action = false
location = "cosmos"
must_use_at_least_once = ["hubris::report"]
role = "coordinator"

[description]
en = "Implement and Execute: draft code, apply changes via file I/O and host commands, track progress, handle failures."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "hubris"
tool_name = "list_todo"

[[related_tools]]
agent_name = "hubris"
tool_name = "create_todo"

[[related_tools]]
agent_name = "polemos"
tool_name = "host_command_exec"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "notify_file_operation"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "unregister_file_operation"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_store"

[[related_tools]]
agent_name = "philia"
tool_name = "memory_query"

[[related_tools]]
agent_name = "aporia"
tool_name = "rag_db_read"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_index"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_status"

[[related_skills]]
agent_name = "skopeo"
tool_name = "context_overflow_handler"

[[related_skills]]
agent_name = "kalos"
tool_name = "file_read"

[[related_skills]]
agent_name = "kalos"
tool_name = "file_write"
+++

# plan_execute

**YOU ARE IN EXECUTION MODE. You MUST call exec with real code. Outputting a plan without exec = FAILURE.**

## Execution Philosophy

**Split-first.** If a task can be decomposed into independent sub-tasks, split it. Use `create_todo` to register sub-tasks, handle the first one yourself, and report the rest for follow-up execution. This is the primary strategy for large work — don't try to do everything in one pass.

**Act, don't just analyze.** After reading code and understanding the problem, you MUST take concrete action before reporting. Action means any of:

- Write files directly (`file_write`)
- Delegate by creating sub-tasks for subagents (`create_todo` + report remaining work)
- Run modifications via shell (`host_command_exec` with `sed`, `git`, etc.)

Reporting "I found issues but didn't fix them" is a FAILURE — at minimum, fix one file or register one TODO.

**Long-term value over quick patches.** Prefer thorough fixes that benefit the codebase long-term over quick patches — unless the user explicitly says "urgent" or "hotfix".

**Be aggressive.** If you see an opportunity to improve the codebase while fixing the requested issue, do it.

**One file at a time.** For multi-file work: read ONE file, fix it, write it, commit, then move to the next. Do NOT read all files first and then try to fix them all.

**Track technical debt.** When you MUST do a temporary fix, register a TODO:

```json
exec({ code: "import { create_todo } from 'hubris'; await create_todo({ title: 'Tech debt: WHAT and WHY', metadata: { tags: ['tech_debt', 'priority:medium'] } }); console.log('registered');" })
```

**Commit your work** after each change:

```json
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cd WS_ROOT && git add -A && git commit -m \"description\"', timeout: 30 }); console.log(r.data?.stdout || r.data?.stderr);" })
```

## Task Classification

Check the user's original message (NOT workplan). If prompt has BOTH analysis verb (find/scan/查) AND fix verb (fix/修复/resolve) → **investigate-and-fix**, NOT read-only.

| Scope | → Action |
| --- | --- |
| Infrastructure / dual-container / multi-step pipeline | Chain shell steps in one exec with `&&` and markers |
| **Investigate-and-fix** (find + fix) | Read code first (evidence required!), then fix. You MUST read ≥1 source file before concluding "no issues". Zero-evidence judgment = FAILURE. |
| Read-only analysis ONLY (no fix verb) | Run clippy/rg, report findings, do NOT modify |
| Automation (clippy fix, format) | `cargo clippy --fix --allow-dirty`, verify, commit |
| Single-file edit | Read→modify→write in ONE exec |
| Multi-file surgery / refactor | Batch modify-verify, `cargo check` after each batch |
| Fallback | Write first, verify after |

Unsure between read-only and fix? → If prompt mentions any fix verb, treat as write.

## Key Rules

1. **Tool selection**: Use `file_read` / `file_write` (kalos basic tools) for ALL file I/O. Do NOT use `smart_read_file` or `smart_write_file` — they require a Cosmos container context and will fail with "requires a Cosmos execution context" error in Scepter local mode. `host_command_exec` (polemos) is for shell commands on the host.
1. **Workspace path discovery**: On your FIRST exec call, run `pwd` to discover where you are. The workspace may be at `/workspace` (inside container) or `/mnt/sdb1/entelecheia` (on host). Use the discovered path for ALL subsequent shell commands. Do NOT guess and retry — check once, then use consistently.

   - `file_read`/`file_write` (kalos tools) always use `/workspace/` prefix — these run inside the container.
   - `host_command_exec` (polemos) runs on the host — use the host workspace path you discovered.

1. **Null-safe**: Always check `r.ok && r.data` before accessing fields. Tool results use `{ ok: boolean, data: T, error: string|null }` format — NOT `success`.
1. **ONE exec per operation**: Chain read→modify→write in a single exec.
1. **Verify after writes**: `cargo check -p PKG` for `.rs` files. Read-back for non-code.
1. **Recent changes**: Before modifying, run `git log --oneline -10` to avoid redoing solved work.
1. **Observer**: Register via `notify_file_operation` before writes, unregister after.
1. **Max retries**: 5 modify-verify cycles for surgery, 3 for auto-fix.

## SoP-6: Self-Surgery Protocol

When the task involves modifying this project's own source code (self-surgery), follow these additional steps on top of the standard Task Classification and Key Rules.

**Pre-surgery (automatic):**

- `PreSurgeryCheckpoint` hook fires before chain start, recording `git rev-parse HEAD` via evernight IPC.
- Use `discover_hooks({ namespace_prefix: "pipeline.surgery" })` to verify safety nets are registered before starting.

**During surgery:**

- Follow the standard modify-verify cycle (Key Rule 7: max 5 cycles).
- After each file write, explicitly call `code_verify` to check affected packages:

  ```json
  exec({ code: "import { code_verify } from 'classic_software_engineering'; const r = await code_verify({ packages: ['PKG1', 'PKG2'] }); console.log(r.data);" })
  ```

**Post-surgery (automatic):**

- `PostSurgeryRollback` hook (priority 80) runs `cargo check --workspace` after chain ends.
  - If check **passes** → `NoaMergeCommit` (priority 50) merges noa workspaces + commits.
  - If check **fails** → rolls back to pre-surgery git ref (`git reset --hard <ref>`), chain aborts.

**Failure handling:**

- If rollback fires, the report MUST include: which package failed, the error message from cargo check, and the reverted commit ref.
- Do NOT retry the same modification that caused a rollback without changing approach.

## MANDATORY: File Write Enforcement

**If the task involves writing or modifying ANY file, you MUST produce an actual file change before calling `report()`.** This is non-negotiable. Generating a plan, writing code to a variable, or describing what you would do is NOT sufficient. The file on disk MUST change.

### Primary method: kalos::file_write

Call `file_write` via exec to write the file:

```js
exec({ code: "import { file_write } from 'kalos'; const r = await file_write({ path: '/workspace/PATH', content: CONTENT }); console.log(r.ok ? 'written' : r.error);" })
```

You MUST check the result: if `r.ok` is true, the file was written. If false, use the fallback below.

### Fallback: host_command_exec (use this if file_write is unclear or fails)

If you are unsure how to call `file_write`, or if it returns an error, use `host_command_exec` to write the file directly via shell. This ALWAYS works:

```js
// For writing content to a file via shell (use heredoc for multi-line):
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'cat > /workspace/PATH << \'ENDOFFILE\'\nYOUR_CONTENT_HERE\nENDOFFILE', timeout: 10 }); console.log(r.data?.stdout || r.data?.stderr || 'done');" })
```

Or for appending a single line:
```js
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'echo \'LINE_CONTENT\' >> /workspace/PATH', timeout: 10 }); console.log(r.data?.stdout || 'done');" })
```

### Verification: confirm the file changed

After writing, ALWAYS verify the change landed on disk:

```js
exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'head -5 /workspace/PATH', timeout: 5 }); console.log(r.data?.stdout || 'empty');" })
```

**ANTI-PATTERN — DO NOT DO THIS:**
- Writing the plan to a `write_to_var` variable and then calling `report()` without any file change
- Calling `file_write` in a try-catch that silently swallows the error
- Describing what you would write instead of actually writing it
- Calling `report()` with "I would write..." or "The change should be..."

**CORRECT PATTERN:**
1. Write the file (via `file_write` OR `host_command_exec`)
2. Verify the file changed (via `host_command_exec` reading the file)
3. Call `report()` with a summary of what was actually written

## Quick Patterns

**Discover workspace**: `host_command_exec({ command: 'pwd && ls Cargo.toml 2>/dev/null && echo FOUND', timeout: 5 })` — if no Cargo.toml, try `cd /mnt/sdb1/entelecheia && pwd`.
**Find files**: `host_command_exec({ command: 'cd WS_ROOT && rg -l "KEYWORD" --type rust | head -20', timeout: 10 })`
**Find files with special chars** (e.g. `$`, `.`): Use `rg -F` for literal strings or single-quote the pattern: `rg -F '$.agent' docs/ | head -20`
**Read file**: `file_read({ path: '/workspace/PATH' })` — check `r.ok && r.data.content`
**Write file**: `file_write({ path: '/workspace/PATH', content: MODIFIED })`
**Shell command**: `host_command_exec({ command: 'cd WS_ROOT && CMD', timeout: N })` — use discovered WS_ROOT
**Verify**: `cargo check -p PKG 2>&1 | tail -10` — check for `error[`
**Revert**: `git checkout -- PATH`
