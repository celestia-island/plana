+++
name = "task_decompose"
agent = "hubris"

[[triggers]]
topic_pattern = "channel.*.message"

[[next_action]]
agent = "hubris"
name = "workplan_generate"

[description]
en = "Unified Requirement Parsing + Recursive Task Decomposition + DAG Construction — the single entry point from natural language to structured task plan."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_index"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_status"

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
agent_name = "hubris"
tool_name = "report_human"

[[related_skills]]
agent_name = "eleos"
tool_name = "web_search_summarize"

[features]
execution_mode = "read"
must_touch_next_action = false
location = "cosmos"
+++

# task_decompose

Parse natural language requirements into structured task definitions, then decompose into an atomic sub-task DAG.

## CRITICAL: Resilience Rules

You MUST follow these resilience rules to avoid stalling the skill chain:

1. **memory_query is OPTIONAL.** If `memory_query` fails (timeout, embedding error, or returns null), do NOT retry, do NOT wait, do NOT report an error. Simply proceed WITHOUT memory context. Memory is a cache for convenience — it is never a prerequisite for task decomposition.

2. **list_todo / create_todo may not be exported.** Always wrap calls to `list_todo` in a try-catch:
   ```js
   exec({ code: "try { const { list_todo } = await import('hubris'); const t = await list_todo({ view: 'tree' }); console.log(JSON.stringify(t)); } catch(e) { console.log('list_todo unavailable, proceeding without TODO survey'); }" })
   ```
   If it throws `could not find export`, skip the survey step entirely and proceed to classification.

3. **NEVER call report_human for operational tasks.** If the user message contains ANY of these signals, it is operational (category 3), NOT conversational:
   - Mentions a file path (e.g., `packages/scepter/src/...`, `*.rs`, `*.md`)
   - Contains code snippets (fenced code blocks, `fn `, `pub `, `use `, `import `)
   - Uses operational verbs: write, modify, edit, fix, add, remove, update, create, delete, refactor
   - References cargo, git, build, compile, test, clippy, fmt
   - Contains `kalos::file_write`, `file_write`, `code_generate`, `plan_execute`
   - The message is from YOLO cruise control (auto_fix, regression_monitor, etc.)

4. **When in doubt, route to plan_execute.** A false positive (sending a conversational message to plan_execute) is recoverable. A false negative (sending an operational task to report_human) loses the work entirely.

## Quick Classify

Before decomposition, assess what kind of message this is:

### 1. Purely conversational / small talk

- Greetings, self-introductions, casual remarks, "how are you", "thank you"
- YOLO cruise control: "start/stop cruise control", "开始/停止自动巡航", "yolo start/stop"

→ `report_human({ summary, body })` only. Do NOT call `report()`. Chain terminates.
**Tone**: Natural, warm, concise. Don't over-answer a casual remark.

### 2. Stance / opinion / analytical questions (NO file operations)

Questions that ask for critique, evaluation, opinion, design analysis, meta-reflection, or open-ended reasoning — where the answer lives in the LLM's reasoning capability, not in files on disk:

- "锐评/评价/分析 这个设计/框架/架构"
- "你觉得这个方案有什么问题"
- "从架构角度看..."
- "另一个 AI 向你提问..." (meta questions)
- Any question where the user is asking for YOUR perspective, not code execution

**Unless the user explicitly says "先读取代码" or "基于事实" or "read the code first"**, do NOT route these to plan_execute. The LLM can reason directly.

If the user DOES ask to read code first → treat as operational (below).

→ Call `report_human` (from orexis module), NOT `report` (from hubris module).
   Using `report()` will incorrectly continue the chain to plan_execute.
   `report_human()` terminates the chain and delivers the answer to the user.

**IMPORTANT**: You MUST actually call `report_human` via `exec`. Writing content
to `write_to_var` is NOT enough — the user will never see it unless you call
`report_human`. A common mistake is to write the answer and then stop without
calling report_human. Don't do this — ALWAYS finish with the exec call.

For SHORT responses (under ~500 chars), skip write_to_var entirely:
```js
exec({ code: "import { report_human } from 'orexis'; report_human({ summary: '短摘要', body: '完整回复内容' });" });
```

For LONGER responses, use write_to_var then immediately call exec:
```js
write_to_var({ var_name: "ans", content: "长篇分析内容..." });
exec({ code: "import { report_human } from 'orexis'; report_human({ summary: '摘要', body: vars['ans'] });" });
```

### 3. Operational (involves file ops, code execution, data manipulation)

- Any task requiring file read/write, code execution, container ops, workspace scanning
- Even simple ones like "list files", "scan workspace", "show me the structure"
- Includes: "先读取代码再回答" (read the code first, then answer)

→ Route to `workplan_generate` → `plan_execute` (which has kalos.smart_read_file,
skemma.script_exec, neikos tools, etc.). Your job is to ROUTE, not execute.

**CRITICAL**: If you're unsure whether a request needs code access, **err on the side of
category 3 (operational)**. Sending an operational task to report_human (category 2)
LOSES THE WORK ENTIRELY — the file never gets written, the chain never reaches code_generate.
A false positive (sending a simple question to plan_execute) is harmless — plan_execute
will just read and respond. When in doubt: ROUTE TO plan_execute.

**NEVER classify a message as conversational (category 1 or 2) if it contains:**
- File paths, code snippets, or programming keywords
- Action verbs: write, modify, edit, fix, add, remove, update, create, delete
- Tool names: kalos, file_write, code_generate, plan_execute, host_command_exec
- Build/cargo/git references
- YOLO skill names: auto_fix, code_generate, code_verify, regression_monitor

## Dependency Impact Assessment

If the system prompt includes a **## Dependency Context** section, use it to:

1. **Mark high-impact tasks**: If a sub-task modifies a package that others depend on (has reverse dependencies), mark it `priority: high` and add `impact: shared-crate` annotation.
1. **Order shared-crate changes first**: Tasks modifying shared crates (`packages/shared/*`) must come before tasks modifying downstream consumers in the DAG.
1. **Add verification sub-tasks**: After any high-impact modification, add an explicit verification step: `cargo check -p <affected-package>`.
1. **Flag cascade risk**: If a task modifies a package with 3+ downstream dependents, note `cascade_risk: true` in the sub-task metadata.

## Fast-Path: Mechanical Fix ONLY

**Match criteria (ALL must be true)**:

1. Task explicitly requests a **mechanical/code-tooling operation**: clippy fix, cargo fmt, rustfmt, unused imports removal, formatting cleanup
1. No ambiguity, no analysis needed, no evaluation required
1. Does NOT involve: code review, quality assessment, critique, analysis, architecture evaluation, self-evaluation, reporting findings

**Keywords that MUST NOT trigger fast-path**: review, critique, evaluate, assess, analyze, 锐评, 评价, 评估, 分析, 审查, 诊断 — these require human-grade analysis. Route to Tier 2 (conversational, report_human) if no code access is needed, or full SoP if the user asks to read code first.

**When matched → SKIP survey and full decomposition.** Immediately report a minimal 1-step plan:

```js
write_to_var({ var_name: "rep", content: "## Task: Auto-fix\n\nMechanical code fix (clippy/format/unused). Single-step execution via plan_execute SoP-2." })
exec({ code: "import { report } from 'hubris'; report({ text: vars['rep'] });" })
```

Do NOT call `list_todo()`. Do NOT call `llm_chat()`. Do NOT decompose. Just report and let `plan_execute` handle it via its SoP.

## SoP

1. **Survey** (OPTIONAL — skip if tools unavailable) — Try to read current TODOs. If `list_todo` is not exported or throws, skip immediately:
   ```js
   exec({ code: "try { const { list_todo } = await import('hubris'); const todos = await list_todo({ view: 'tree' }); console.log(JSON.stringify(todos, null, 2)); } catch(e) { console.log('list_todo unavailable, skipping survey'); }" })
   ```
1. **Scope Probe** — For tasks involving specific patterns, identifiers, or file types, run a quick search to estimate actual scope before classifying:

   ```js
   exec({ code: "import { host_command_exec } from 'polemos'; const r = await host_command_exec({ command: 'rg -c \"PATTERN\" --type rust --type md 2>/dev/null | wc -l', cwd: '<host-workspace>', timeout: 10 }); console.log('match_count:', r.data?.stdout?.trim() || '0');" })
   ```

   Replace `<host-workspace>` with the host path from the environment section's `Workspace:` line (strip the `local://` prefix, e.g. `local:///mnt/codespace` → `/mnt/codespace`). Pass it as the `cwd` parameter — never hardcode a path and never `cd` inside the command string (the shell's working directory is not the workspace).

   Use the probe to inform decomposition:

   - 0 matches → the pattern may not exist; report this finding rather than assuming work to do
   - 1–5 matches → likely single-file or small edit
   - 5–50 matches → multi-file surgery; break into batches per directory or language
   - 50+ matches → large-scale update; decompose into per-directory sub-tasks, prioritize by impact

1. **Parse** — Extract intent, entities, constraints. Flag ambiguities. Use scope probe results to set realistic expectations. **Prioritize long-term improvements over temporary patches** — if the task can be solved thoroughly vs quickly, decompose for the thorough path unless user explicitly says "urgent"/"紧急".
1. **Decompose** — Break into 2–10 atomic sub-tasks. Annotate each: `execution_mode` (read|write|edge), `parallelizable` (true if read-only + independent), `priority` (high|medium|low). If scope probe revealed large-scale work, note `scope: large` in the DAG metadata so `plan_execute` selects the appropriate execution strategy.
1. **Map DAG** — Express dependencies. Detect circular deps. Mark parallelizable batches.
1. **Report** — MUST call `report()`:

   ```js
   write_to_var_json({ var_name: "dag", content: { intent, subtasks: [...], ... } })
   write_to_var({ var_name: "rep", content: "## Task Decomposition\n\n### Intent\n...\n### Sub-task DAG\n..." })
   exec({ code: "import { report } from 'hubris'; let _r = {}; _r.text = vars['rep']; report(_r); _r.text" })
   ```

> Return type and IEPL enforcement: @system/return-type-convention
