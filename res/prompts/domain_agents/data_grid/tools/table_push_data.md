+++
name = "table_push_data"
agent = "data_grid"

[description]
en = "Push row data into a table widget (full or incremental)"
zh-Hans = "向表格控件推送行数据（全量或增量）"
+++

# table_push_data

## Description

Broadcasts a `Sync.ViewDataPush` via `panel_push_data` with the typed
`fields`/`records` payload. `full_replace=false` merges incrementally so
agents can stream rows without clobbering user edits.

## Parameters

- `layout_id`: target panel instance id
- `widget_id`: table widget id
- `data`: `{ fields: [...], records: [...] }` or partial
- `full_replace`: boolean (default true)

## Example

```json
{
  "layout_id": "tab-1",
  "widget_id": "table-main",
  "data": {
    "fields": [{ "key": "value", "type": "number" }],
    "records": [{ "id": "row-4", "cells": { "value": 55 } }]
  },
  "full_replace": false
}
```
