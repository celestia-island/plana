+++
name = "model_place"
agent = "digital_twin"

[description]
en = "Place or move a 3D model in the twin scene (world coordinates)"
zh-Hans = "在孪生场景中放置或移动 3D 模型（世界坐标）"
+++

# model_place

## Description

Creates or updates a DeviceModel in the twin scene. Uses
`projects.deviceModels.create` / `projects.deviceModels.update`
to persist name, GLB, world position, rotation, scale and the
polemos telemetry binding.

## Parameters

- `model_id`: existing model id (update) or omitted (create)
- `name`: display name
- `glb_url`: GLB key (e.g. "box1_enc.glb")
- `position`: `{x, y, z}` world coordinates (metres)
- `rotation`: `{x, y, z}` radians
- `scale`: uniform scale
- `polemos_node_id`: telemetry station binding, or null

## Example

```json
{
  "model_id": "a0000001-0000-4000-0000-000000000001",
  "name": "Compressor Enclosure",
  "position": { "x": 27.11, "y": 1.76, "z": 24.86 },
  "scale": 1.0,
  "polemos_node_id": "comp-enc"
}
```
