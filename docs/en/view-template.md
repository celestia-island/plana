# The panel view-template format

Status: **active** (the format lives in `plana-celestia-types` 0.1.1, module
`ws::ui::view_template`; published bindings `@celestia-island/plana-types` 0.1.9;
中文版：[view-template.md](../zh-Hans/view-template.md))

Audience: template authors and the hosts that load them

A workspace panel is configured by a **template**: a declarative document
that names the base view the panel is drawn on and fills that base's spec.
The format lives in `plana-celestia-types` (`ws::ui::view_template`) and is
exported to TypeScript as `@celestia-island/plana-types`, so the same shapes
reach a Rust host and the web UI.

Nothing in the format **executes code**. A template can only combine
vocabulary the reading build already draws. The types put no whitelist on the
engine id — a host reports an engine it cannot draw as unsupported rather than
silently substituting one.

## The five base views

Every template builds on exactly one of these, and sets exactly the matching
spec arm:

| Base | What it draws | Spec |
|---|---|---|
| `waterfall` | A bucketed card stream over independent columns | `WaterfallSpec` |
| `node-canvas` | A pannable, zoomable node-and-edge surface | `NodeCanvasSpec` |
| `data-grid` | A typed tabular grid | `DataGridSpec` |
| `kanban` | Lanes of draggable typed cards, with an optional card-style lane header | `KanbanSpec` |
| `blank-canvas` | A full-bleed mount the host's renderer fills | `BlankCanvasSpec` |

`kanban` is deliberately NOT a waterfall variant: a waterfall enumerates one
stream into columns, while a board is a set of lanes whose cards move between
them. The chat node list and the multidimensional table's board view are
both specialisations of the board.

## The document

```toml
version = 1

[[template]]
id = "reports"
title = "Reports"
title_key = "reports.title"
render_engine = "waterfall-stream"

[template.spec]
base = "waterfall"

[template.spec.waterfall]
columns = { kind = "adaptive", max = 2, min_width = 320 }

[template.spec.waterfall.grouping]
field = "timestamp"
newest_first = true

[template.spec.waterfall.windowing]
source = "client"
estimated_item_height = 150
overscan_screens = 1.0

[[template.spec.chrome.docks]]
widget = "chat-bar"
placement = "bottom"
anchor = "plane"
surface = "glass"
```

- `version` is a number. A reader that does not know the version must
  **reject** the file rather than guess — but that check belongs to the host:
  `validate()` never looks at `version`.
- `title` is required; `title_key` is an optional i18n key the shell
  translates instead of `title`. `description` and `description_key` are the
  same optional pair for the panel's description.
- `id` and `render_engine` must each match `^[a-z][a-z0-9-]*$`. That is the
  whole rule: there is no engine whitelist and no `celestia-` prefix
  convention — `render_engine` names whatever plugin the reading host's
  registry resolves.
- A `[[template]]` entry sets **exactly** the spec arm its `base` names.
- Chrome hangs off the **spec**, not the template, and docks are a *list* of
  contributions: each one is a `[[template.spec.chrome.docks]]` table. A
  `[template.chrome.docks]` table is dropped without an error — the types
  accept unknown fields — so the template still validates and the panel just
  renders with no dock.

## Fields worth knowing

**Waterfall** — `columns` (single / fixed / adaptive), optional `grouping`
(the day buckets), optional `windowing` (client- or host-owned; absent = the
whole list is rendered), the `card_kinds` it may render, and optional
`rail` / `search` widgets.

**Data grid** — `fields` (the field-schema ids the grid binds to),
`editable` (in-place row editing, `false` by default), and an optional
`overlay` widget drawn above the grid.

**Kanban** — `axes` (`horizontal` / `vertical` / `both`; the lane axis and
the card axis are independent), `lanes` (the field the lane key comes from,
and whether the lane header is itself drawn as a card — `header_is_card`
defaults to `false`), `card_kinds`, the `dnd` rules (`across_lanes` and
`reorder`, both `false` by default), and optional `windowing` for long
boards.

**Node canvas** — `backend` (`svg` / `dom-overlay-svg` / `canvas-edges-dom` /
`blank`), `camera` bounds, the `node_kinds` it may render, and optional
`palette` / `inspector` rails and an `auto_layout` algorithm id. The minimap
is an **optional table**: leave the `minimap` key out and none is
configured; when the key is present its `enabled` and `placement` default to
`true` and `bottom-right`.

**Blank canvas** — `mount` (the renderer id the host resolves for the
full-bleed mount) and an optional `overlay` drawn above it.

**Docks** — `DockPlacement` covers all eight anchors: the four edges
(`top` / `bottom` / `left` / `right`, centred) and the four corners, each on
the `page` or `plane` plane with a `glass` or `solid` finish. A contribution's
`anchor`, `surface` and `order` default to `page`, `glass` and `0`. Some hosts
draw fewer: a host must reject a placement it cannot draw, never ignore it.

## Validation

Loading is two steps, and the second one is not optional:

```rust
let file: ViewTemplateFile = toml::from_str(&text)?;
file.validate()?; // id/engine/key shapes, the arm matching the base, numeric bounds
```

`PanelTemplate::validate` checks everything the types cannot — the `id` and
`render_engine` shapes (`^[a-z][a-z0-9-]*$`), the i18n-key shapes, that
**exactly the arm named by `base`** is set, and the numeric bounds (a column
count of at least one, a camera needing a finite `0 < min_zoom <= max_zoom`
and a positive `zoom_step`). `ViewTemplateFile::validate` runs it over every
template in the file, so one broken template fails the whole file.

It does **not** read `version`: the number is carried and never checked, so
rejecting a file whose `version` the reader does not know is a check the host
has to make for itself.

A host that skips validation gets a renderer that ignores what it cannot
draw — the failure mode the format exists to avoid.
