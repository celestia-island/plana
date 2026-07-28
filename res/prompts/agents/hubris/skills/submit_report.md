+++
name = "submit_report"
agent = "hubris"
status = "convention"

[description]
en = "[CONVENTION] — The report_human pattern. This is a documentation convention, not an executable skill."
zhs = "[约定] — report_human 模式。这是文档约定，不是可执行技能。"

[features]
execution_mode = "read"
location = "cosmos"
+++

# submit_report → CONVENTION (not a skill)

This is a **documentation convention**, not an executable skill. All skills that need to report to humans should use the `report_human` pattern directly.

## report_human Convention

When any skill needs to deliver a report to the human user via `report_human()`:

**Always use BOTH `summary` and `body`** — `summary` is a concise one-line summary (required), `body` is the full detailed report in Markdown.

**Step 1**: Store the concise summary:

```json
write_to_var({ var_name: "reply_summary", content: "One-line summary of what was accomplished" })
```

**Step 2**: Store the full detailed body:

```json
write_to_var({ var_name: "reply_body", content: "## Details\n\nFull detailed report in Markdown..." })
```

**Step 3**: Assemble both and call in `exec`:

```typescript
exec({ code: "import { report_human } from 'hubris'; import vars from 'vars'; report_human({ summary: vars['reply_summary'], body: vars['reply_body'] });" })
```

**For very short replies** (single-line greeting, quick answer), you may omit `body` — but still name the variable clearly:

```text
write_to_var({ var_name: "reply_summary", content: "Hello! I'm HubRis, the intelligent decision engine..." })
exec({ code: "import { report_human } from 'hubris'; import vars from 'vars'; report_human({ summary: vars['reply_summary'] });" })
```

**Rules**:

- `summary` (required): Concise one-line summary of what was accomplished. Always include this. **MUST be written in the user's preferred language** (from `env.aporia.language` or `user_language`/`preferred_language` in system prompt).
- `body` (optional): Full detailed report in Markdown. Include for any non-trivial output. Must also use the user's preferred language.
- NEVER write the report to the filesystem — always use `write_to_var` + `exec`
- ALL user-facing text (`summary`, `body`) MUST be in the user's preferred language — do NOT default to English unless the system prompt explicitly sets it.

This convention is now part of the mcp documentation. Do NOT invoke `submit_report` as a skill — just follow the pattern inline.
