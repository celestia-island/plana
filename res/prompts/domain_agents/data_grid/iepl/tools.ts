/**
 * Data Grid agent IEPL tools — drive multidimensional tables.
 *
 * Underlying MCP tools are exposed to the exec sandbox as named imports
 * from this agent's bridge (`import { panel_push_data } from
 * 'data_grid'` …); this file re-exports them with table-domain
 * signatures for skills.
 */

// ── Table mutations (via widget.edit — validated on scepter) ─────────────

/** Add a row to a table widget. */
export async function table_add_row(params: {
  layout_id: string;
  widget_id: string;
  row_id: string;
  cells: Record<string, unknown>;
}): Promise<{ ok: boolean }> {
  const result = await widget_edit({
    layoutId: params.layout_id,
    widgetId: params.widget_id,
    kind: "table.add_row",
    payload: { row_id: params.row_id, cells: params.cells },
  });
  return { ok: result?.ok !== false };
}

/** Update a single cell. */
export async function table_update_cell(params: {
  layout_id: string;
  widget_id: string;
  row_id: string;
  field: string;
  value: unknown;
}): Promise<{ ok: boolean }> {
  const result = await widget_edit({
    layoutId: params.layout_id,
    widgetId: params.widget_id,
    kind: "table.update_cell",
    payload: { row_id: params.row_id, field: params.field, value: params.value },
  });
  return { ok: result?.ok !== false };
}

/** Delete a row. */
export async function table_delete_row(params: {
  layout_id: string;
  widget_id: string;
  row_id: string;
}): Promise<{ ok: boolean }> {
  const result = await widget_edit({
    layoutId: params.layout_id,
    widgetId: params.widget_id,
    kind: "table.delete_row",
    payload: { row_id: params.row_id },
  });
  return { ok: result?.ok !== false };
}

// ── Data push (Sync.ViewDataPush) ────────────────────────────────────────

/** Push row data into a table widget (full or incremental). */
export async function table_push_data(params: {
  layout_id: string;
  widget_id: string;
  data: Record<string, unknown>;
  full_replace?: boolean;
}): Promise<{ ok: boolean }> {
  const result = await panel_push_data({
    layout_id: params.layout_id,
    widget_id: params.widget_id,
    data: params.data,
    full_replace: params.full_replace ?? true,
  });
  return { ok: result?.ok !== false };
}
