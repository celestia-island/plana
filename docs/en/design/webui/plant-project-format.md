+++
title = "Plant Project File Format (`.plant.json`)"
description = "Project file format design — similar to Siemens TIA Portal project files, unifying industrial node topology, 2D panels, and 3D scenes."
lang = "en"
category = "design"
subcategory = "webui"
+++

# Plant Project File Format (`.plant.json`)

> Project file format design — similar to Siemens TIA Portal project files, unifying industrial node topology, 2D panels, and 3D scenes.

## Design Goals

1. **Single source of truth** — One file describes the entire plant/project: device nodes, 2D topology, 3D scene, industrial networks
1. **Three-way compatibility** — `mock_scepter` (fixture), shittim-chest webui (3D rendering), entelecheia PoleMos agent (device management) all read the same file
1. **Node-centric** — All topology, scene, and sensor data is attached to nodes; the node is the core entity
1. **Versionable** — `format_version` field + JSON Schema for backward-compatible evolution
1. **Extensible** — Custom metadata can be appended without breaking existing parsers

## File Conventions

- Extension: `.plant.json`
- Encoding: UTF-8
- Format: JSON (natively supported by all three parties)
- One file = one project = one plant/production line

## Top-level Structure

```json
{
  "$schema": "https://shittim-chest.ai/schemas/plant-v1.json",
  "format_version": 1,

  "metadata": { ... },
  "nodes": { ... },
  "topology": { ... },
  "scene": { ... }
}
```

---

## Section 1: `metadata`

Project metadata.

```json
{
  "metadata": {
    "name": "Green Hydrogen Corridor",
    "description": "Green hydrogen corridor demonstration project",
    "author": "engineering-team",
    "created_at": "2026-06-13T00:00:00Z",
    "updated_at": "2026-06-13T00:00:00Z",
    "tags": ["hydrogen", "green-energy", "demo"]
  }
}
```

Field descriptions:

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| name | string | Y | Project name |
| description | string | N | Description |
| author | string | N | Creator |
| created_at | ISO8601 | N | Creation time |
| updated_at | ISO8601 | N | Last modified time |
| tags | string[] | N | Tags |

---

## Section 2: `nodes`

**Core entity**. Each node represents a physical device or logical unit. Other sections (topology, scene) reference nodes by ID.

```json
{
  "nodes": {
    "rsoc-enc": {
      "label": "RSOC Enclosure",
      "label_i18n": { "zhs": "RSOC 系统外壳" },
      "type": "rsoc",
      "box": "box-1",
      "polemos_node_id": "node-rsoc-enc",
      "manufacturer": "Example Corp",
      "model": "RSOC-2024-ENC",
      "serial": "SN-RSOCENC-012345",
      "rated": {
        "rated_power": "150 kW",
        "operating_temperature": "600 ~ 850 °C",
        "fuel": "H₂ / CH₄",
        "generation_efficiency": "≥ 60%"
      },
      "sensors": [
        {
          "id": "tt-101",
          "type": "temperature",
          "label": "TT-101",
          "address": "Modbus:HR101",
          "unit": "°C",
          "range": [0, 1000]
        },
        {
          "id": "pt-101",
          "type": "pressure",
          "label": "PT-101",
          "address": "Modbus:HR102",
          "unit": "MPa",
          "range": [0, 5]
        }
      ],
      "status": "online",
      "metadata": {}
    },

    "rsoc-stack": {
      "label": "RSOC Stack",
      "type": "rsoc-stack",
      "box": "box-1",
      "polemos_node_id": "node-rsoc-stack",
      "rated": {},
      "sensors": [],
      "status": "online"
    }
  }
}
```

Field descriptions:

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| label | string | Y | Display name |
| label_i18n | {lang: string} | N | Multilingual names |
| type | string | Y | Device type identifier (rsoc / pem / tank / compressor / fuelcell / synthesis / chp / structure / ...) |
| box | string | Y | Enclosure ID, corresponds to topology.boxes[].id |
| polemos_node_id | string | N | entelecheia PoleMos agent node ID for cloud-edge collaboration |
| manufacturer | string | N | Manufacturer |
| model | string | N | Model |
| serial | string | N | Serial number |
| rated | {key: string} | N | Nameplate parameters |
| sensors | Sensor[] | N | Associated sensors |
| status | string | N | Default status (online / offline / maintenance) |
| metadata | object | N | Extension fields |

Sensor structure:

| Field | Type | Description |
| --- | --- | --- |
| id | string | Sensor ID (e.g. tt-101) |
| type | string | temperature / pressure / flow / level / gas / current |
| label | string | Display label |
| address | string | Industrial protocol address (Modbus:HR101 / OPC-UA:ns=2;s=Temperature) |
| unit | string | Unit |
| range | [min, max] | Measurement range |

---

## Section 3: `topology`

2D panel topology — used for SCADA-style panel views and 2D pipeline diagrams.

```json
{
  "topology": {
    "boxes": [
      {
        "id": "box-1",
        "label": "#1 RSOC System",
        "label_i18n": { "zhs": "#1 RSOC 系统", "en": "#1 RSOC System" },
        "color": "#8b5cf6",
        "nodes": ["rsoc-enc", "rsoc-stack"]
      },
      {
        "id": "box-2",
        "label": "#2 Electrolyzer Area",
        "label_i18n": { "zhs": "#2 电解槽区", "en": "#2 Electrolyzer Area" },
        "color": "#3b82f6",
        "nodes": ["alk2", "alk3", "bop", "pem", "pem-cluster"]
      }
    ],

    "plcs": [
      { "id": "plc-central", "label": "PLC-Central", "ip": "192.168.10.40", "protocol": "Modbus TCP" }
    ],

    "connections": {
      "signal_wires": [
        {
          "id": "sw-rsoc-1",
          "from": "tt-101",
          "to": "plc-central",
          "protocol": "Modbus",
          "points": [[260,130],[150,130],[150,500],[60,500]]
        }
      ],
      "power_cables": [
        {
          "id": "pc-rsoc-1",
          "from": "mcc-panel",
          "to": "rsoc-enc",
          "voltage": "380V",
          "points": [[500,50],[300,200]]
        }
      ],
      "water_pipes": [
        {
          "id": "wp-rsoc-1",
          "from": "cooling-tower",
          "to": "rsoc-enc",
          "medium": "circulating cooling water",
          "points": [[50,400],[300,300]],
          "flow_rate": 120,
          "temperature": 28
        }
      ],
      "gas_pipes": [
        {
          "id": "gp-rsoc-1",
          "from": "rsoc-enc",
          "to": "h2-manifold",
          "gas": "H2",
          "points": [[390,250],[600,250]]
        }
      ]
    },

    "layout": {
      "rsoc-enc": { "pos": [300, 200], "size": [180, 100] },
      "tt-101":   { "pos": [260, 130] },
      "pt-101":   { "pos": [340, 130] },
      "plc-central": { "pos": [60, 500] }
    }
  }
}
```

Topology field descriptions:

| Field | Type | Description |
| --- | --- | --- |
| boxes | Box[] | Enclosure groupings; each box contains several nodes |
| plcs | PLC[] | PLC device list |
| connections | Connections | Four connection types: signal wires, power cables, water pipes, gas pipes |
| layout | {id: LayoutItem} | 2D panel coordinates (2D position of each node / sensor / plc) |

Box structure:

| Field | Type | Description |
| --- | --- | --- |
| id | string | Enclosure ID |
| label | string | Display label |
| label_i18n | {lang: string} | Multilingual |
| color | string | Theme color |
| nodes | string[] | List of contained node IDs |

Connection structure (common):

| Field | Type | Description |
| --- | --- | --- |
| id | string | Connection ID |
| from | string | Source entity ID (node / sensor / plc / utility) |
| to | string | Destination entity ID |
| points | [x,y][] | Polyline path coordinates |
| protocol | string | Signal wire protocol (Modbus / 4-20mA / Profibus / HART / OPC-UA) |
| voltage | string | Power cable voltage |
| medium | string | Water pipe medium |
| gas | string | Gas pipe type |
| flow_rate | number | Flow rate |
| temperature | number | Temperature |

---

## Section 4: `scene`

3D holographic scene configuration — used by webui `PhysicalPreview`'s Three.js rendering.

```json
{
  "scene": {
    "background_color": "#0a0a1a",
    "environment_url": null,

    "camera": {
      "overview": {
        "position": [10, 15, 50],
        "target": [10, 2, 20],
        "fov": 45
      },
      "bookmarks": {
        "box-1": { "position": [28, 6, 32], "target": [27, 2, 24] },
        "box-2": { "position": [23, 6, 35], "target": [18, 1, 27] },
        "box-3": { "position": [10, 6, 37], "target": [6, 2, 27] },
        "box-4": { "position": [2, 6, 35], "target": [-1, 2, 26] },
        "box-5": { "position": [-2, 6, 29], "target": [-5, 2, 21] },
        "box-6": { "position": [-5, 6, 16], "target": [-8, 2, 8] },
        "box-7": { "position": [14, 6, 16], "target": [15, 2, 7] }
      }
    },

    "lighting": {
      "ambient": {
        "color": [0.70, 0.75, 0.82],
        "intensity": 500
      },
      "directional": {
        "color": [0.90, 0.92, 0.95],
        "intensity": 6000,
        "position": [15, 80, 30]
      }
    },

    "ground": {
      "enabled": false
    },

    "bloom": {
      "strength": 0.5,
      "radius": 0.4,
      "threshold": 0.85
    },

    "models": [
      {
        "node": "rsoc-enc",
        "glb": "box1_rsoc_enclosure.glb",
        "position": [27.11, 1.76, 24.86],
        "rotation": [0, 0, 0],
        "scale": 1.0,
        "material": "auto",
        "is_background": false
      },
      {
        "node": "rsoc-stack",
        "glb": "box1_rsoc_stack.glb",
        "position": [28.39, 1.62, 23.05],
        "rotation": [0, 0, 0],
        "scale": 1.0,
        "material": "auto",
        "is_background": false
      }
    ]
  }
}
```

Scene field descriptions:

| Field | Type | Description |
| --- | --- | --- |
| background_color | string | 3D scene background color |
| environment_url | string? | HDR environment map URL |
| camera.overview | CameraView | Initial camera angle (overview of all models) |
| camera.bookmarks | {boxId: CameraView} | Fly-to target angle for each enclosure |
| lighting.ambient | {color, intensity} | Ambient light |
| lighting.directional | {color, intensity, position} | Directional light |
| ground | {enabled, ...} | Ground configuration |
| bloom | {strength, radius, threshold} | Bloom post-processing |
| models | Model3D[] | 3D model placement list |

CameraView structure:

| Field | Type | Description |
| --- | --- | --- |
| position | [x, y, z] | Camera position |
| target | [x, y, z] | Look-at target point |
| fov | number | Field of view (degrees) |

Model3D structure:

| Field | Type | Description |
| --- | --- | --- |
| node | string | Associated node ID |
| glb | string | GLB filename (relative to models/ directory) |
| position | [x, y, z] | 3D world coordinates |
| rotation | [x, y, z] | Euler angle rotation |
| scale | number | Scale factor |
| material | "auto" \| "holographic" \| "native" | Material override |
| is_background | boolean | Whether this is a background decoration |

---

## Three-Party Integration

### 1. mock_scepter (Rust fixture)

Load path: `fixtures/{project}.plant.json`

```text
fixtures/
├── agents.json
├── devices.json
├── hydrogen_corridor.plant.json   ← new
└── models/
    ├── box1_rsoc_enclosure.glb
    ├── box2_alk2.glb
    └── ...
```

On `mock_scepter` startup:

- `fixtures::load_all()` gets a new `load_plant()` call
- Parse `.plant.json` → split into `DeviceModelResponse[]` + `SceneConfigItem`
- `get_scene_config` returns from plant data instead of hardcoded values
- `list_device_models` returns from plant data
- `topology.rs`'s `box_detail()` / `equipment_detail()` derived from plant data

### 2. shittim-chest webui

Existing API contracts remain unchanged (`/projects/{pid}/device-models` + `/projects/{pid}/device-models/scene-config`).

New additions:

- `PhysicalPreview.tsx`'s `BOX_CAMERA_TARGETS` reads from `scene.camera.bookmarks` instead of being hardcoded
- 3D model CSS2D overlay labels read from `nodes[nodeId].label`

### 3. entelecheia PoleMos agent

PoleMos reads the plant file via MCP tools:

- `node_discover` → iterate `nodes` + `topology.plcs`
- `device_self_test` → read `nodes[id].sensors` + `nodes[id].rated`
- Device management operations → write back `nodes[id].status`

Future extensions:

- PoleMos layer2 agent generates `.plant.json` (AI reads device docs to automatically build topology)
- Users drag-and-drop 3D layout in webui → write back `.plant.json`
- CI/CD pipeline validates `.plant.json` schema integrity

---

## Example File

See `scripts/mock/fixtures/hydrogen_corridor.plant.json` for a complete example (to be created).

---

## Relationship to Existing Data

| Existing data source | Portion migrated to .plant.json |
| --- | --- |
| `http_server.rs` hardcoded 20 DeviceModelResponses | → `scene.models[]` + `nodes{}` |
| `http_server.rs` hardcoded SceneConfigItem | → `scene{}` (camera, lighting, ground, bloom) |
| `mock_data/topology.rs` overview() / box_detail() | → `topology{}` (boxes, connections, layout) |
| `mock_data/topology.rs` equipment_detail() | → `nodes{}.rated` + `nodes{}.sensors` |
| `PhysicalPreview.tsx` BOX_CAMERA_TARGETS | → `scene.camera.bookmarks` |
| `devices.json` fixture (entelecheia PoleMos) | → `nodes{}.polemos_node_id` |

## Schema Validation

The JSON Schema file lives at `schemas/plant-v1.json` and is shared by all three parties.
`mock_scepter` validates via `serde_json` deserialization + schema validation at load time.
The webui can validate using `ajv` at build time.
entelecheia can validate using the `jsonschema` crate.
