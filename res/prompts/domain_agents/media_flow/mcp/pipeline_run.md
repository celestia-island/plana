+++
name = "pipeline_run"
agent = "media_flow"

[description]
en = "Run or update a media generation pipeline (image/3D/audio/video)"
zh-Hans = "运行或更新媒体生成管线（图像/3D/音频/视频）"
+++

# pipeline_run

## Description

Pushes a pipeline definition to the media-flow panel via
`panel_push_layout` (Sync.DashboardLayoutPush with a `node-graph`
widget) so the user can review the graph, or drives the existing
pipeline through the `media.*` endpoints.

## Parameters

- `layout_id`: target panel instance id
- `pipeline`: `{ nodes: [...], edges: [...] }` — MediaNode/MediaEdge graph

## Nodes

`prompt`, `reference_image`, `text_to_image`, `image_to_3d`,
`text_to_3d`, `render_scene`, `vision_critique`, `refine_code`,
`mesh_optimize`, `pbr_texture`, `loop_control`, `export_glb`,
`register_model`.

## Example

```json
{
  "layout_id": "pipe-1",
  "pipeline": {
    "nodes": [{ "id": "n1", "type": "prompt", "label": "Prompt",
                "x": 40, "y": 200, "params": { "text": "hydrogen pump" } }],
    "edges": []
  }
}
```
