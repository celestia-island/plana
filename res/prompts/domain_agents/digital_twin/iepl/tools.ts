/**
 * Digital Twin agent IEPL tools — drive the 3D holographic panel.
 *
 * The underlying MCP tools are exposed to the exec sandbox as named
 * imports from this agent's bridge (`import { scene_push } from
 * 'digital_twin'` …); this file re-exports them with panel-domain
 * signatures so skills can call them idiomatically.
 *
 * Remote target: the chest backend (panel_push_* broadcast →
 * Sync.DashboardLayoutPush; deviceModels.* RPCs).
 */

// ── Scene control ───────────────────────────────────────────────────────

/** Push a full scene/layout to the holographic panel. */
export async function scene_push(params: {
  layout_id: string;
  layout: Record<string, unknown>;
}): Promise<{ ok: boolean }> {
  const result = await panel_push_layout({
    layout_id: params.layout_id,
    layout: params.layout,
  });
  return { ok: result?.ok !== false };
}

// ── Model placement ──────────────────────────────────────────────────────

/** Place or move a 3D model in the twin scene. */
export async function model_place(params: {
  model_id?: string;
  name: string;
  glb_url?: string;
  position?: { x: number; y: number; z: number };
  rotation?: { x: number; y: number; z: number };
  scale?: number;
  polemos_node_id?: string;
}): Promise<{ model_id: string }> {
  const method = params.model_id ? "projects.deviceModels.update" : "projects.deviceModels.create";
  const body = { id: params.model_id, name: params.name, glb_url: params.glb_url, position: params.position, rotation: params.rotation, scale: params.scale, polemos_node_id: params.polemos_node_id };
  const result = await exec(method, body);
  return { model_id: String(result?.model_id ?? result?.id ?? "") };
}

/** Read the twin scene configuration. */
export async function scene_config_get(params?: { projectId?: string }): Promise<Record<string, unknown>> {
  return (await exec("projects.sceneConfig.get", { projectId: params?.projectId })) ?? {};
}

/** Update the twin scene configuration. */
export async function scene_config_update(params: {
  projectId?: string;
  background_color?: string;
  ground?: Record<string, unknown>;
  camera?: Record<string, unknown>;
}): Promise<{ ok: boolean }> {
  const result = await exec("projects.sceneConfig.update", params);
  return { ok: result?.ok !== false };
}
