+++
name = "table_edit"
agent = "data_grid"

[description]
en = "Mutate a multidimensional table (add/update/delete rows, update fields)"
zh-Hans = "修改多维表（增删行、更新单元格与字段）"
+++

# table_edit

## Description

Sends a `widget.edit` request with a `table.*` kind to the panel agent
router. The scepter validates the payload; the client applies the edit
optimistically and rolls back on rejection.

## Kinds

- `table.add_row` — payload `{ row_id, cells }`
- `table.update_cell` — payload `{ row_id, field, value }`
- `table.delete_row` — payload `{ row_id }`
- `table.update_field` — payload `{ field, ... }`

## Parameters

- `layout_id`: target panel instance id
- `widget_id`: table widget id
- `kind`: one of the table.* kinds above
- `payload`: kind-specific fields

## Example

```json
{
  "layout_id": "tab-1",
  "widget_id": "table-main",
  "kind": "table.update_cell",
  "payload": { "row_id": "row-3", "field": "value", "value": 42.5 }
}
```
