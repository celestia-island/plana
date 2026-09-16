# The panel view-template format

> Status: active (the format shipped in `plana-celestia-types` 0.1.9)
> Audience: template authors and the hosts that load them

A workspace panel is configured by a **template**: a declarative document
that names the base view the panel is drawn on and fills that base's spec.
The format lives in `plana-celestia-types` (`ws::ui::view_template`) and is
exported to TypeScript as `@celestia-island/plana-types`, so the same shapes
reach a Rust host and the web UI.

Nothing in the format **executes code**. A template can only combine
vocabulary the reading build already draws; an engine it does not know is
reported as unsupported rather than silently replaced.

## The five base views

Every template builds on exactly one of these, and sets exactly the matching
spec arm:

| Base | What it draws | Spec |
|---|---|---|
| `waterfall` | A bucketed card stream over independent columns | `WaterfallSpec` |
| `node-canvas` | A pannable, zoomable node-and-edge surface | `NodeCanvasSpec` |
| `data-grid` | A typed tabular grid | `DataGridSpec` |
| `kanban` | Lanes of draggable typed cards, each lane headed by a card | `KanbanSpec` |
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
render_engine = "celestia-waterfall"

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

[template.chrome.docks]
placement = "bottom"
anchor = "plane"
surface = "glass"
widget = "chat-bar"
```

- `version` is a number. A reader that does not know the version must
  **reject** the file rather than guess.
- `id`, `render_engine` and the i18n keys follow the same shapes the
  workspace-module contract fixes; `render_engine` names the plugin that
  draws the template.
- A `[[template]]` entry sets **exactly** the spec arm its `base` names.

## Fields worth knowing

**Waterfall** — `columns` (single / fixed / adaptive), optional `grouping`
(the day buckets), optional `windowing` (client- or host-owned), the
`card_kinds` it may render, and optional `rail` / `search` widgets.

**Kanban** — `axes` (`horizontal` / `vertical` / `both`; the lane axis and
the card axis are independent), `lanes` (the field the lane key comes from,
and whether the lane header is itself a card), `card_kinds`, and the `dnd`
rules.

**Node canvas** — `backend` (`svg` / `dom-overlay-svg` / `canvas-edges-dom` /
`blank`), `camera` bounds, and a minimap that is **on by default** in the
bottom-right corner.

**Docks** — `DockPlacement` covers all eight anchors: the four edges
(`top` / `bottom` / `left` / `right`, centred) and the four corners, each on
the `page` or `plane` plane with a `glass` or `solid` finish. Some hosts draw
fewer: a host must reject a placement it cannot draw, never ignore it.

## Validation

Loading is two steps, and the second one is not optional:

```rust
let file: ViewTemplateFile = toml::from_str(&text)?;
file.validate()?; // id/key shapes, the arm matching the base, numeric bounds
```

`PanelTemplate::validate` checks everything the types cannot — the id and
i18n-key shapes, that **exactly the arm named by `base`** is set, and the
numeric bounds (a column count of at least one, a camera whose zoom range is
finite and ordered). `ViewTemplateFile::validate` runs it over every template
in the file, so one broken template fails the whole file.

A host that skips validation gets a renderer that ignores what it cannot
draw — the failure mode the format exists to avoid.
