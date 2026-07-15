+++
id = "mcp_brief"
title = "MCP 工具调用速查"
kind = "reference"
+++

# MCP Tool Calling — Quick Reference

## The Two-Step Pattern (ALWAYS use this for output)

You MUST deliver your output via `report()`. Follow this exact pattern:

```json
Step 1: write_to_var({ var_name: "rep", content: "...your text..." })
Step 2: exec({ code: "import { report } from 'hubris'; let _rpt = {}; _rpt.text = vars['rep']; report(_rpt); _rpt.text" })
```

For JSON data, use `write_to_var_json` in Step 1.

**This is MANDATORY** — your output will be LOST if you do not call `report()`.
Plain text output without `report()` is NOT captured by the pipeline.

## Critical Rules

1. **ES module imports** — Use `import { tool } from 'agent'` syntax inside `exec` code:

   - `import { report } from 'hubris';` ✓
   - `import { list_todo } from 'hubris';` ✓
   - `report(...)` — also works if imported, but explicit import is preferred

1. **Parameters MUST be JS objects, NEVER raw JSON strings** — MCP tools accept structured JS objects, not serialized strings:

   - `report({ text: "result" })` ✓
   - `report('{"text":"result"}')` ✗ (raw JSON string — will fail)

1. **Await all tool calls** — Every tool call returns a Promise. Use `await`:

   ```typescript
   const todos = await list_todo({ view: 'tree' });
   ```

1. **Unique variable names per exec** — NEVER reuse `let`/`const` names across `exec()` calls:

   ```json
   exec({ code: "let r1 = {}; ..." })   // first call
   exec({ code: "let r2 = {}; ..." })   // second call — different name
   ```

1. **Last expression = return value** — The final expression in exec is captured. For reports, end with `_rpt.text`.

1. **`write_to_var` is for DATA, NOT CODE** — Never store JavaScript code in a variable and try to eval/exec it. Import and call tools directly in `exec`.

## Tool Result Format

Every tool call returns `{ ok: boolean, data: any, error: string | null }`.
Always `await` and check `.ok` before using `.data`.
