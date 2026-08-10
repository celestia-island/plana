+++
name = "read_iteration_plan"
agent = "hubris"

[[next_action]]
agent = "hubris"
name = "task_decompose"

[description]
en = "Read the architecture truth table and parse the Iteration Backlog into a structured task list — the internal entry point for the pure self-bootstrap loop."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "list_todo"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_status"

[[related_skills]]
agent_name = "hubris"
tool_name = "task_decompose"

[[related_skills]]
agent_name = "kalos"
tool_name = "file_read"

[features]
execution_mode = "read"
must_touch_next_action = false
location = "cosmos"
must_use_at_least_once = ["hubris::report"]
+++

# read_iteration_plan

Parse the **Iteration Backlog** table from the architecture truth table into a
structured, machine-readable task list. This skill is the **internal entry
point** of the pure self-bootstrap loop: once it returns a structured list,
`task_decompose` → `workplan_generate` → `plan_execute` can take over without
relying on an external agent platform to read the backlog.

## Why this skill exists

Before IB-01, the YOLO loop worked only because an external agent platform
(opencode / Claude Code / Cursor) was reading `architecture.md` and
decomposing backlog items itself. The planner/dispatcher role lived outside
Entelecheia. To close the self-bootstrap gap, Entelecheia's own coordinator
must be able to do this parsing internally — that's what this skill is for.

## SoP

**You are in PLANNING mode.** This skill is read-only: it does NOT modify
files, do not call `file_write`, do not call `host_command_exec`. Your job is
to **observe** the architecture truth table and **report** the structured
result via `report()`.

1. **Locate the truth table.** The Iteration Backlog lives in
   `docs/en/designs/architecture.md` (relative to the entelecheia worktree
   root). Use `file_read({ path: '/workspace/docs/en/designs/architecture.md' })`
   inside the Cosmos container. If `workspace_status` reports a different
   workspace root, adjust the path accordingly. The container path is
   `/workspace/...`, the host path is the discovered `WS_ROOT`.

1. **Locate the table.** Search for the markdown section heading
   `### Iteration Backlog`. The table immediately follows. The table has
   five columns in this order:

   | Column | Meaning |
   | ------ | ------- |
   | ID     | Backlog identifier (e.g. `IB-01`, `IB-04`) |
   | Title  | Short task name; the title cell is the canonical name (e.g. `` `hubris::read_iteration_plan` skill ``) |
   | Status | One of: `pending`, `in_progress`, `partial`, `done (via third-party driver)`, `superseded`, or a free-form descriptor prefixed with `done` |
   | Acceptance Criteria | Multi-line criteria describing "done" for the row |
   | Notes | Free-form prose; may include cross-references, blockers, history |

1. **Parse each row.** For every table row:

   - Extract `id` as a string (e.g. `"IB-01"`).
   - Extract `title` as a string. Strip backticks but keep the semantic name.
   - Normalize `status` into one of four machine states:
     - `pending` — row is `pending`
     - `in_progress` — row is `in_progress` (this includes the row for THIS skill while it's being implemented)
     - `partial` — row is `partial`
     - `done` — row starts with `done` (covers `done (via third-party driver)`)
     - `superseded` — row is `superseded` (kept for archaeology; not actionable)
   - Keep `acceptance` as the raw text of the Acceptance Criteria cell.
   - Keep `notes` as the raw text of the Notes cell.

1. **Filter actionable rows.** Discard rows whose normalized status is
   `done`, `superseded`, or `partial` UNLESS this skill is the active
   orchestrator. The orchestrator MAY include `partial` rows when the
   `notes` field declares a follow-up. The output is the actionable subset.

1. **Order rows.** Within the actionable subset, sort by:
   1. Status priority: `in_progress` first, then `pending`, then `partial` (only if included).
   1. ID lexicographic ascending (`IB-01` before `IB-02`).

1. **Report the structured list.** You MUST call `report()` with a JSON
   payload of the form:

   ```js
   write_to_var({ var_name: "backlog", content: JSON.stringify({
     source: "docs/en/designs/architecture.md#iteration-backlog",
     parsed_at: "<ISO-8601 UTC>",
     total_rows: <int>,
     actionable_count: <int>,
     items: [
       {
         id: "IB-01",
         title: "hubris::read_iteration_plan skill",
         status: "in_progress",
         priority: "P0",
         acceptance: "<raw acceptance cell>",
         notes: "<raw notes cell>"
       }
     ]
   }, null, 2) })
   exec({ code: "import { report } from 'hubris'; report({ text: vars['backlog'] });" })
   ```

   The report's `text` field is the canonical payload that downstream
   `task_decompose` consumes.

1. **Stop.** Do NOT proceed to `task_decompose` yourself — that is the
   `next_action` declared in the frontmatter and the orchestrator will
   trigger it. Your single deliverable is the parsed, sorted, actionable
   backlog list delivered via `report()`.

## Failure modes

| Symptom | Likely cause | Recovery |
| ------- | ------------ | -------- |
| `file_read` returns `r.ok = false` | Workspace not mounted; wrong path | Run `workspace_status` and `pwd` to discover `WS_ROOT`; retry with `/workspace/<rel>` |
| `### Iteration Backlog` heading not found | Architecture doc moved; this skill ran on a fork that renamed the section | Stop and report the failure via `report()`; do NOT guess |
| A row's `Status` cell has no recognizable token | The cell is a free-form descriptor (e.g. "blocked on …") | Default to `pending` and surface a `warnings` array in the report payload |
| Table contains fewer than 5 columns | Doc was edited and lost structure | Stop and report the malformed table |

## Anti-patterns

- **Do NOT** modify `architecture.md`. This skill is read-only by design.
  Status updates are the responsibility of `task_decompose` → the orchestrator
  → a follow-up skill (IB-04 backlog status auto-update will formalize this).
- **Do NOT** call `task_decompose` directly. The orchestrator owns
  `next_action` chaining. A double-trigger causes duplicate plan generation.
- **Do NOT** invent or hallucinate backlog items. If a row is missing from
  the table, the report's `total_rows` will be lower than expected — that is
  the signal, not a license to add.
- **Do NOT** include `done` / `superseded` rows in `items`. They inflate the
  plan and pollute downstream `task_decompose`.

## Examples

### Happy path

```text
Input:  `architecture.md` Iteration Backlog has 10 rows.
Steps:
  1. file_read('docs/en/designs/architecture.md') → full doc.
  2. Locate `### Iteration Backlog`; extract the 10 rows.
  3. Normalize statuses; filter actionable.
  4. Report the structured JSON via report().
Output: actionable_count = 5 (IB-01, IB-04, IB-06, IB-07, IB-09 say)
```

### Partial row inclusion

```text
Input:  IB-03 is `partial`; its notes say "task-level criteria not yet".
Steps:
  1. Parse all rows; IB-03 normalizes to `partial`.
  2. Active orchestrator flag is on; include IB-03 in items.
  3. The notes become a `follow_up` field in the report payload.
Output: actionable_count = 6, IB-03 included with follow_up: "task-level criteria"
```

### Architecture doc moved

```text
Input:  `### Iteration Backlog` not found.
Steps:
  1. file_read succeeded; doc has 600+ lines.
  2. Heading search returned no match.
  3. report({ text: JSON.stringify({ error: "section not found", heading_search: "### Iteration Backlog" }) })
Output: actionable_count = 0, error surfaced.
```

> Return type and IEPL enforcement: @system/return-type-convention
