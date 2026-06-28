+++
title = "Plant 專案檔案格式 (`.plant.json`)"
description = """> 工程檔案格式設計——類似西門子博圖 (TIA Portal) 的工程檔案，統一描述工業節點拓撲、2D 面板、3D 場景。"""
lang = "zht"
category = "design"
subcategory = "webui"
+++

# Plant 專案檔案格式 (`.plant.json`)

> 工程檔案格式設計——類似西門子博圖 (TIA Portal) 的工程檔案，統一描述工業節點拓撲、2D 面板、3D 場景。

## 設計目標

1. **單一資料來源**——一個檔案描述整個工廠/專案：設備節點、2D 拓撲、3D 場景、工業網路
1. **三方相容**——`mock_scepter` (fixture)、shittim-chest webui (3D 渲染)、entelecheia PoleMos agent (設備管理) 都讀同一份檔案
1. **節點為中心**——所有拓撲、場景、感測器都掛在 node 上，node 是核心實體
1. **可版本管理**——`format_version` 欄位 + JSON Schema，支援向後相容演進
1. **可擴展**——允許追加自訂 metadata 不破壞現有解析器

## 檔案約定

- 後綴：`.plant.json`
- 編碼：UTF-8
- 格式：JSON（三端都原生支援）
- 每個檔案 = 一個 project = 一個工廠/產線

## 頂層結構

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

工程元資料。

```json
{
  "metadata": {
    "name": "Green Hydrogen Corridor",
    "description": "氫能走廊示範工程",
    "author": "engineering-team",
    "created_at": "2026-06-13T00:00:00Z",
    "updated_at": "2026-06-13T00:00:00Z",
    "tags": ["hydrogen", "green-energy", "demo"]
  }
}
```

欄位說明：

| 欄位 | 類型 | 必填 | 說明 |
| --- | --- | --- | --- |
| name | string | Y | 工程名稱 |
| description | string | N | 描述 |
| author | string | N | 建立者 |
| created_at | ISO8601 | N | 建立時間 |
| updated_at | ISO8601 | N | 最後修改時間 |
| tags | string[] | N | 標籤 |

---

## Section 2: `nodes`

**核心實體**。每個 node 代表一個物理設備/邏輯單元。其他 section（topology、scene）透過 node ID 引用。

```json
{
  "nodes": {
    "rsoc-enc": {
      "label": "RSOC Enclosure",
      "label_i18n": { "zhs": "RSOC 系統外殼" },
      "type": "rsoc",
      "box": "box-1",
      "polemos_node_id": "node-rsoc-enc",
      "manufacturer": "Example Corp",
      "model": "RSOC-2024-ENC",
      "serial": "SN-RSOCENC-012345",
      "rated": {
        "額定功率": "150 kW",
        "工作溫度": "600 ~ 850 °C",
        "燃料": "H₂ / CH₄",
        "發電效率": "≥ 60%"
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

欄位說明：

| 欄位 | 類型 | 必填 | 說明 |
| --- | --- | --- | --- |
| label | string | Y | 顯示名稱 |
| label_i18n | {lang: string} | N | 多語言名稱 |
| type | string | Y | 設備類型標識 (rsoc / pem / tank / compressor / fuelcell / synthesis / chp / structure / ...) |
| box | string | Y | 所屬箱體 ID，對應 topology.boxes[].id |
| polemos_node_id | string | N | entelecheia PoleMos agent 的節點 ID，用於雲邊協同 |
| manufacturer | string | N | 廠商 |
| model | string | N | 型號 |
| serial | string | N | 序列號 |
| rated | {key: string} | N | 銘牌參數 |
| sensors | Sensor[] | N | 關聯感測器 |
| status | string | N | 預設狀態 (online / offline / maintenance) |
| metadata | object | N | 擴展欄位 |

Sensor 結構：

| 欄位 | 類型 | 說明 |
| --- | --- | --- |
| id | string | 感測器 ID (如 tt-101) |
| type | string | temperature / pressure / flow / level / gas / current |
| label | string | 顯示標籤 |
| address | string | 工業協定位址 (Modbus:HR101 / OPC-UA:ns=2;s=Temperature) |
| unit | string | 單位 |
| range | [min, max] | 量程 |

---

## Section 3: `topology`

2D 面板拓撲——用於 SCADA 式面板視圖、2D 管線圖。

```json
{
  "topology": {
    "boxes": [
      {
        "id": "box-1",
        "label": "#1 RSOC 系統",
        "label_i18n": { "zhs": "#1 RSOC 系統", "en": "#1 RSOC System" },
        "color": "#8b5cf6",
        "nodes": ["rsoc-enc", "rsoc-stack"]
      },
      {
        "id": "box-2",
        "label": "#2 電解槽區",
        "label_i18n": { "zhs": "#2 電解槽區", "en": "#2 Electrolyzer Area" },
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
          "medium": "循環冷卻水",
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

topology 欄位說明：

| 欄位 | 類型 | 說明 |
| --- | --- | --- |
| boxes | Box[] | 箱體分組，每個 box 包含若干 node |
| plcs | PLC[] | PLC 設備列表 |
| connections | Connections | 四類連接：訊號線、電力電纜、水管、氣管 |
| layout | {id: LayoutItem} | 2D 面板座標 (每個 node / sensor / plc 的 2D 位置) |

Box 結構：

| 欄位 | 類型 | 說明 |
| --- | --- | --- |
| id | string | 箱體 ID |
| label | string | 顯示標籤 |
| label_i18n | {lang: string} | 多語言 |
| color | string | 主題色 |
| nodes | string[] | 包含的 node ID 列表 |

Connection 結構（通用）：

| 欄位 | 類型 | 說明 |
| --- | --- | --- |
| id | string | 連線 ID |
| from | string | 起點實體 ID (node / sensor / plc / utility) |
| to | string | 終點實體 ID |
| points | [x,y][] | 折線路徑座標 |
| protocol | string | 訊號線協定 (Modbus / 4-20mA / Profibus / HART / OPC-UA) |
| voltage | string | 電力電纜電壓 |
| medium | string | 水管介質 |
| gas | string | 氣管氣體類型 |
| flow_rate | number | 流量 |
| temperature | number | 溫度 |

---

## Section 4: `scene`

3D 全息場景設定——用於 webui `PhysicalPreview` 的 Three.js 渲染。

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

scene 欄位說明：

| 欄位 | 類型 | 說明 |
| --- | --- | --- |
| background_color | string | 3D 場景背景色 |
| environment_url | string? | HDR 環境貼圖 URL |
| camera.overview | CameraView | 初始攝影機視角（正對模型全景） |
| camera.bookmarks | {boxId: CameraView} | 每個箱體的飛入目標視角 |
| lighting.ambient | {color, intensity} | 環境光 |
| lighting.directional | {color, intensity, position} | 方向光 |
| ground | {enabled, ...} | 地面設定 |
| bloom | {strength, radius, threshold} | 泛光後處理 |
| models | Model3D[] | 3D 模型放置列表 |

CameraView 結構：

| 欄位 | 類型 | 說明 |
| --- | --- | --- |
| position | [x, y, z] | 攝影機位置 |
| target | [x, y, z] | 注視目標點 |
| fov | number | 視場角 (度) |

Model3D 結構：

| 欄位 | 類型 | 說明 |
| --- | --- | --- |
| node | string | 關聯的 node ID |
| glb | string | GLB 檔名 (相對於 models/ 目錄) |
| position | [x, y, z] | 3D 世界座標 |
| rotation | [x, y, z] | 歐拉角旋轉 |
| scale | number | 縮放 |
| material | "auto" \| "holographic" \| "native" | 材質覆蓋 |
| is_background | boolean | 是否為背景裝飾物 |

---

## 三端對接方案

### 1. mock_scepter (Rust fixture)

載入路徑：`fixtures/{project}.plant.json`

```text
fixtures/
├── agents.json
├── devices.json
├── hydrogen_corridor.plant.json   ← 新增
└── models/
    ├── box1_rsoc_enclosure.glb
    ├── box2_alk2.glb
    └── ...
```

`mock_scepter` 啟動時：

- `fixtures::load_all()` 新增 `load_plant()` 呼叫
- 解析 `.plant.json` → 拆分為 `DeviceModelResponse[]` + `SceneConfigItem`
- `get_scene_config` 從 plant 資料返回，不再硬編碼
- `list_device_models` 從 plant 資料返回
- `topology.rs` 的 `box_detail()` / `equipment_detail()` 從 plant 資料衍生

### 2. shittim-chest webui

現有 API 契約不變（`/projects/{pid}/device-models` + `/projects/{pid}/device-models/scene-config`）。

新增：

- `PhysicalPreview.tsx` 的 `BOX_CAMERA_TARGETS` 從 `scene.camera.bookmarks` 讀取，不再硬編碼
- 3D 模型的 CSS2D overlay 標籤從 `nodes[nodeId].label` 讀取

### 3. entelecheia PoleMos agent

PoleMos 透過 MCP tool 讀取 plant 檔案：

- `node_discover` → 走訪 `nodes` + `topology.plcs`
- `device_self_test` → 讀取 `nodes[id].sensors` + `nodes[id].rated`
- 設備管理操作 → 寫回 `nodes[id].status`

未來可擴展：

- PoleMos layer2 agent 生成 `.plant.json`（透過 AI 讀取設備文件自動建立拓撲）
- 人在 webui 裡拖曳編輯 3D 佈局 → 寫回 `.plant.json`
- CI/CD 管線驗證 `.plant.json` 的 schema 完整性

---

## 範例檔案

完整範例見 `scripts/mock/fixtures/hydrogen_corridor.plant.json`（待建立）。

---

## 與現有資料的關係

| 現有資料來源 | 遷移到 .plant.json 的部分 |
| --- | --- |
| `http_server.rs` 硬編碼的 20 個 DeviceModelResponse | → `scene.models[]` + `nodes{}` |
| `http_server.rs` 硬編碼的 SceneConfigItem | → `scene{}` (camera, lighting, ground, bloom) |
| `mock_data/topology.rs` 的 overview() / box_detail() | → `topology{}` (boxes, connections, layout) |
| `mock_data/topology.rs` 的 equipment_detail() | → `nodes{}.rated` + `nodes{}.sensors` |
| `PhysicalPreview.tsx` 的 BOX_CAMERA_TARGETS | → `scene.camera.bookmarks` |
| `devices.json` fixture (entelecheia PoleMos) | → `nodes{}.polemos_node_id` |

## Schema 校驗

JSON Schema 檔案放在 `schemas/plant-v1.json`，三端共享。
`mock_scepter` 載入時 `serde_json` 反序列化 + schema 校驗。
webui 建置時可用 `ajv` 校驗。
entelecheia 可用 `jsonschema` crate 校驗。
