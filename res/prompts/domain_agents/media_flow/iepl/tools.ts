/**
 * Media Flow agent IEPL tools — drive media generation pipelines.
 *
 * Underlying MCP tools are exposed to the exec sandbox as named imports
 * from this agent's bridge (`import { media_call } from 'media_flow'`
 * …); this file re-exports them with pipeline-domain signatures.
 */

// ── Pipeline control ─────────────────────────────────────────────────────

/** Push a pipeline definition to the media-flow panel. */
export async function pipeline_push(params: {
  layout_id: string;
  pipeline: { nodes: unknown[]; edges: unknown[] };
}): Promise<{ ok: boolean }> {
  const result = await panel_push_layout({
    layout_id: params.layout_id,
    layout: {
      title: "Media Pipeline",
      widgets: [{ id: "pipeline-main", type: "node-graph", span: "full", data: params.pipeline }],
    },
  });
  return { ok: result?.ok !== false };
}

// ── Direct media calls (chest media.* RPCs) ──────────────────────────────

/** Call a media generation endpoint directly. */
export async function media_call(params: {
  method: "media.llm_chat" | "media.gen_image" | "media.gen_3d" | "media.register_model";
  payload: Record<string, unknown>;
}): Promise<Record<string, unknown>> {
  return (await exec(params.method, params.payload)) ?? {};
}

// ── Widget mutations (Sync.ViewInstancePush) ─────────────────────────────

/** Create/update/delete a widget on a media-flow panel. */
export async function widget_mutate(params: {
  layout_id: string;
  op: "create" | "update" | "delete";
  widget: Record<string, unknown>;
}): Promise<{ ok: boolean }> {
  const result = await panel_push_instance({
    layout_id: params.layout_id,
    op: params.op,
    widget: params.widget,
  });
  return { ok: result?.ok !== false };
}
