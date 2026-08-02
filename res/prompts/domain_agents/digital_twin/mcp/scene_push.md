+++
name = "scene_push"
agent = "digital_twin"

[description]
en = "Push a full 3D scene/layout update to the holographic panel"
zh-Hans = "将完整 3D 场景/布局推送到全息面板"
+++

# scene_push

## Description

Broadcasts a full dashboard layout to the webui holographic twin via
`panel_push_layout` (Sync.DashboardLayoutPush). The `layout` body carries
the scene descriptor the frontend applies wholesale.

## Parameters

- `layout_id`: target panel instance id
- `layout`: `{ title, subtitle?, widgets: [...] }` — full descriptor

## Example

```json
{
  "layout_id": "holo-1",
  "layout": {
    "title": "Hydrogen Corridor",
    "widgets": [{ "id": "w1", "type": "node-graph", "span": "full" }]
  }
}
```
