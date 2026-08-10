+++
name = "widget_mutate"
agent = "media_flow"

[description]
en = "Create/update/delete a widget on a media-flow panel"
+++

# widget_mutate

## Description

Broadcasts a `Sync.ViewInstancePush` via `panel_push_instance` with
op `create` / `update` / `delete` and the widget descriptor — used to
add a preview pane, swap node labels, or remove stale outputs.

## Parameters

- `layout_id`: target panel instance id
- `op`: "create" | "update" | "delete"
- `widget`: widget descriptor `{ id, type, title?, source, span?, ... }`

## Example

```json
{
  "layout_id": "pipe-1",
  "op": "create",
  "widget": { "id": "preview-1", "type": "node-graph", "source": "media-template", "span": "full" }
}
```
