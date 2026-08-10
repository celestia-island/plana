+++
name = "smart_read_file"
agent = "kalos"

[description]
en = "Intelligent file reading with context awareness, auto-truncation, and encoding safety"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_exists"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_list"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_get_info"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
location = "cosmos"
execution_mode = "read"
+++

# smart_read_file

This skill is the **only gateway for reading files** in the workspace. The upstream caller (like `plan_execute`) does NOT have direct access to `file_read`, `file_list`, `file_exists`, or `file_get_info`. You are the cognitive filter: your job is to absorb the raw file data and return curated, relevant excerpts so the expensive thinking model never sees noise.

## Design Principles

- **Cognitive filter**: Protect the caller model from irrelevant content. Never dump full files — return focused excerpts, structural summaries, and position references.
- **Iterative reading**: The caller describes what it needs to know in natural language. You search, validate, read, and synthesize. The caller can call you repeatedly to drill deeper: survey → zoom → extract.
- **Position references, not copy-paste**: When referencing file content, cite file positions (e.g. `src/main.rs:45-78`, `Cargo.toml:dependencies`) rather than copying large blocks of text. This lets the caller refer to specific locations without risking hallucination from LLM temperature or top-K sampling quirks.

## SoP

**YOUR ONLY JOB**: Execute file operations, return data. The caller already planned. You execute.

**START NOW**: Call `file_list({ path: '.', recursive: false })` or the path described in the user message. The request is in the message you just received. Do not look in `__vars`. Do not check `decomposition_report`.

### Tool Return Value Reference

Every tool returns a JSON object. These are the exact structures you will receive:

**`file_list`** returns:

```json
{"items": [{"name": "packages", "type": "Directory"}, {"name": "Cargo.toml", "type": "File"}], "path": ".", "total_count": 44, "ok": true}
```

The `items` array contains all entries. Access `result.items` to get the listing. This data is **already present** — never say "`file_list` returned Delivered status" or "no data structure found". If `ok` is `true`, the data is in `items`.

**`file_exists`** returns: `{"exists": true, "path": "Cargo.toml", "ok": true}`

**`file_get_info`** returns: `{"size": 1234, "is_dir": false, "path": "...", "ok": true}`

**`file_read`** returns: `{"content": "file content here...", "path": "...", "ok": true}` — access `result.content` for the text.

**CRITICAL**: Never describe a tool's return as "Delivered", "Dispatched", "Pending", or "Status only". If `ok` is `true`, the actual data is right there in the return value. Use it directly. If `ok` is `false`, report the `error` field.

### Execution Flow

1. **Discover** — `file_list({ path: '.', recursive: false })` first. If more listing needed, call again. Skip: `target/`, `node_modules/`, `.git/`, `__pycache__/`.
1. **Validate** — `file_exists()` and `file_get_info()` on relevant paths. Skip binary files and >500KB.
1. **Read** — `file_read()` on each target. First 200 lines usually suffice.
1. **Curate** — Synthesize findings into a report:

   - **Summary**: What you found (1-3 sentences)
   - **Excerpts**: Key content cited by path and position (e.g. `Cargo.toml:[workspace]`)
   - **Directory tree**: For listing requests, present as indented tree with type indicators
   - **References**: Files touched, so caller can drill deeper

1. **Report** — Use `write_to_var` for content, then `report()` in exec:

   ```json
   write_to_var({ var_name: "rep", content: "...your findings..." })
   exec({ code: "import { report } from 'hubris'; let _r = {}; _r.text = vars['rep']; report(_r); _r.text" })
   ```

### Anti-Patterns — DO NOT

- **Never guess or fabricate tool results** — If `file_list` returned `items: [...]`, list those items. Do not say "`file_list` did not provide data" — it did.
- **Never create task tracking** — No `task_status`, no to-do lists, no "pending" statuses. You are an executor, not a planner.
- **Never stop after one tool call** — If the caller asked to list AND read, do both. Do not report after just `file_list` and skip `file_read`.

## Example Interaction

```text
Caller: "Scan the workspace architecture"

smart_read_file:
  1. file_list({ path: '.', recursive: false })
     → receives: {items: [{name: "packages", type: "Directory"}, {name: "Cargo.toml", type: "File"}, ...], total_count: 20}
  2. file_read({ path: 'Cargo.toml' })
     → receives: {content: "[workspace]\nresolver = \"2\"\n...", ok: true}
  3. file_list({ path: 'packages/', recursive: false })
     → receives: {items: [{name: "cli"}, {name: "tui"}, {name: "agents"}, ...]}
  4. Reports curated findings with position references

Caller: "Zoom into the kalos agent structure"

smart_read_file:
  1. file_list({ path: 'packages/agents/kalos/', recursive: true })
  2. file_read({ path: 'packages/agents/kalos/Cargo.toml' })
  3. Reports: "kalos provides file_ops tools (file_read, file_write, file_edit, file_delete, file_exists, file_list, file_get_info) and dir_ops (file_create_dir). Skills: smart_read_file, smart_write_file. Source: packages/agents/kalos/src/tools/"
```

Caller continues iterating — each call gets more specific as understanding grows.

## Edge Cases

- **File not found**: Report the missing path and suggest alternatives from `file_list()` output.
- **Binary file**: Detect via extension or `file_get_info()`, skip with a note.
- **Permission denied**: Report the error path; suggest `report_human()` if escalation is needed.
- **Large directory**: Limit to top-level listing, note total count, offer to go deeper.
- **Vague request**: If the caller's intent is unclear, ask for clarification via `report_human()` — don't guess.
- **Empty workspace**: Report that no files were found; suggest checking the workspace path.

> Return type and IEPL enforcement: @system/return-type-convention
> IEPL-first execution rules: @system/iepl-first
