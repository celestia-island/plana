+++
name = "update_backlog_status"
agent = "hubris"

[description]
en = "Update the Status column of a specific Iteration Backlog row in the architecture truth table — the write-back half of the self-bootstrap loop (IB-04)."
zhs = "更新架构真值表中某个 Iteration Backlog 行的 Status 列 — 自举循环的写回半部分（IB-04）。"
zht = "更新架構真值表中某個 Iteration Backlog 行的 Status 欄位 — 自舉迴圈的寫回半部分（IB-04）。"
ja = "アーキテクチャ真値表の特定の Iteration Backlog 行の Status 列を更新 — 自己ブートストラップループの書き戻し半分（IB-04）。"
ko = "아키텍처 진실 표의 특정 Iteration Backlog 행의 Status 열을 업데이트 — 자체 부트스트랩 루프의 쓰기 반쪽（IB-04）."
fr = "Mettre à jour la colonne Status d'une ligne spécifique de l'Iteration Backlog dans la table de vérité de l'architecture — la moitié écriture du cycle d'auto-amorçage (IB-04)."
es = "Actualizar la columna Status de una fila específica del Iteration Backlog en la tabla de verdad de la arquitectura — la mitad de escritura del bucle de autoarranque (IB-04)."
ru = "Обновить колонку Status конкретной строки Iteration Backlog в таблице истины архитектуры — возвратная половина цикла саморазвёртывания (IB-04)."

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

[[related_tools]]
agent_name = "aporia"
tool_name = "workspace_status"

[[related_skills]]
agent_name = "hubris"
tool_name = "read_iteration_plan"

[features]
execution_mode = "write"
must_touch_next_action = false
location = "cosmos"
must_use_at_least_once = ["hubris::report"]
+++

# update_backlog_status

Update the **Status** column of a specific Iteration Backlog row in
`docs/en/designs/architecture.md`. This skill is the **write-back half** of
the self-bootstrap loop: after a successful chain + commit, the coordinator
dispatches this skill to mark the completed backlog item's status as
`done` (or `in_progress` / `partial` / `blocked`).

## Why this skill exists

Before IB-04, the backlog table's `status` column was edited manually by a
human or the external agent platform driver. This meant that even after a
successful YOLO cycle, the loop could not self-update — the next iteration
would re-discover the same "pending" item because nobody had marked it done.
This skill closes that gap by giving the coordinator an internal mechanism
to write back status updates.

## SoP

**You are in WRITE mode.** This skill modifies `architecture.md`. Use
`file_read` to load the current content, locate the target row, update the
Status column, and use `file_write` to persist the change.

1. **Receive the target.** The coordinator passes:
   - `backlog_id` — e.g. `"IB-01"`, `"IB-04"`
   - `new_status` — one of: `pending`, `in_progress`, `partial`, `done`,
     `superseded`, `blocked`
   - `notes_append` (optional) — a short clause to append to the Notes cell

2. **Locate the truth table.** Use `file_read({ path:
   '/workspace/docs/en/designs/architecture.md' })` to load the full
   document. If `workspace_status` reports a different workspace root,
   adjust the path accordingly.

3. **Find the target row.** Search for a markdown table row whose first
   cell (after the leading `|`) matches `backlog_id` exactly. The row
   pattern is:

   ```markdown
   | IB-01 | `hubris::read_iteration_plan` skill | **in_progress** | ... | ... |
   ```

   The ID cell is `| IB-01 |` (with optional whitespace). Match case-
   sensitively. If no row matches, report the failure and stop — do NOT
   create new rows.

4. **Update the Status column.** The Status column is the **third** column
   (index 2, zero-based). Replace its content with the `new_status`,
   wrapped in `**...**` for bold formatting (matching the existing
   convention). For example:

   - Old: `| IB-04 | Backlog status auto-update | pending | ... |`
   - New: `| IB-04 | Backlog status auto-update | **in_progress** | ... |`

   If `notes_append` is provided, append it to the **fifth** column (Notes,
   index 4). Prefix with a space. Do not remove existing Notes content.

5. **Write the file back.** Use `file_write({ path:
   '/workspace/docs/en/designs/architecture.md', content: <updated> })`
   to persist the change. The content is the full document with the single
   row modified.

6. **Report the result.** Call `report()` with a JSON payload:

   ```js
   exec({ code: "import { report } from 'hubris'; report({ text: JSON.stringify({ backlog_id: 'IB-04', previous_status: 'pending', new_status: 'in_progress', notes_appended: true, file: 'docs/en/designs/architecture.md' }) });" })
   ```

7. **Stop.** Do NOT chain to another skill. The coordinator owns
   `next_action` dispatch.

## Failure modes

| Symptom | Likely cause | Recovery |
| ------- | ------------ | -------- |
| `file_read` returns error | Workspace not mounted; wrong path | Run `workspace_status` to discover `WS_ROOT`; retry |
| Target row not found | `backlog_id` is malformed or the row was deleted | Report failure; do NOT create new rows |
| Multiple rows match same ID | Table is corrupted (duplicate IDs) | Report failure; do NOT modify either row |
| `file_write` returns error | Permission denied; disk full | Report failure; the coordinator may retry or fall back to manual edit |
| Status cell has unexpected format | The row was edited by a human with non-standard formatting | Normalize to `**<status>**` and proceed |

## Anti-patterns

- **Do NOT** create new backlog rows. This skill only updates existing rows.
  New rows are a human decision.
- **Do NOT** modify the Title, Acceptance Criteria, or Notes (beyond the
  optional append) columns. Only the Status column and optionally the Notes
  column are in scope.
- **Do NOT** update multiple rows in one invocation. One `backlog_id` per
  call. The coordinator can dispatch multiple invocations in sequence.
- **Do NOT** delete or reorder rows. The table structure is immutable;
  only cell content changes.
- **Do NOT** use this skill on tables other than the Iteration Backlog.
  Other markdown tables in architecture.md are not in scope.

## Examples

### Mark IB-01 as done

```text
Input:  backlog_id = "IB-01", new_status = "done", notes_append = "Skill doc committed (arona@54830ee); skills.rs registered (entelecheia@b12dcf53c); cargo check verified 2026-07-15."
Steps:
  1. file_read('docs/en/designs/architecture.md') → full doc.
  2. Find row: | IB-01 | `hubris::read_iteration_plan` skill | **in_progress** | ...
  3. Replace **in_progress** → **done**.
  4. Append notes to the Notes cell.
  5. file_write('docs/en/designs/architecture.md', <updated>).
  6. report({ text: JSON.stringify({ backlog_id: 'IB-01', previous_status: 'in_progress', new_status: 'done', ... }) }).
Output: architecture.md IB-01 row now shows **done**.
```

### Mark IB-04 as in_progress

```text
Input:  backlog_id = "IB-04", new_status = "in_progress", notes_append = "BacklogStatusUpdate hook wired in surgery_hooks.rs; update_backlog_status skill doc landed."
Steps:
  1. file_read → full doc.
  2. Find row: | IB-04 | Backlog status auto-update | pending | ...
  3. Replace pending → **in_progress**.
  4. Append notes.
  5. file_write → persist.
  6. report → result.
Output: architecture.md IB-04 row now shows **in_progress**.
```

### Row not found

```text
Input:  backlog_id = "IB-99", new_status = "done"
Steps:
  1. file_read → full doc.
  2. Search for "| IB-99 |" — no match.
  3. report({ text: JSON.stringify({ error: "row not found", backlog_id: 'IB-99' }) }).
Output: No file modification. Error surfaced.
```

> Return type and IEPL enforcement: @system/return-type-convention
