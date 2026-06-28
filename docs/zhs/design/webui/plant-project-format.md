+++
title = "工厂项目文件格式（`.plant.json`）"
description = """> 工程文件格式设计 — 类似西门子博图 (TIA Portal) 的工程文件，统一描述工业节点拓扑、2D 面板、3D 场景。"""
lang = "zhs"
category = "design"
subcategory = "webui"
+++

# 工厂项目文件格式（`.plant.json`）

> 工程文件格式设计 — 类似西门子博图 (TIA Portal) 的工程文件，统一描述工业节点拓扑、2D 面板、3D 场景。

## 设计目标

1. **单一数据源** — 一个文件描述整个工厂/项目：设备节点、2D 拓扑、3D 场景、工业网络
1. **三方兼容** — `mock_scepter` (fixture)、shittim-chest webui (3D 渲染)、entelecheia PoleMos agent (设备管理) 都读同一份文件
1. **节点为中心** — 所有拓扑、场景、传感器都挂在 node 上，node 是核心实体
1. **可版本管理** — `format_version` 字段 + JSON Schema，支持向后兼容演进
1. **可扩展** — 允许追加自定义 metadata 不破坏现有解析器

## 文件约定

- 后缀：`.plant.json`
- 编码：UTF-8
- 格式：JSON（三端都原生支持）
- 每个文件 = 一个 project = 一个工厂/产线

## 顶层结构

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

## Section 1：`metadata`

工程元数据。

```json
{
  "metadata": {
    "name": "Green Hydrogen Corridor",
    "description": "氢能走廊示范工程",
    "author": "engineering-team",
    "created_at": "2026-06-13T00:00:00Z",
    "updated_at": "2026-06-13T00:00:00Z",
    "tags": ["hydrogen", "green-energy", "demo"]
  }
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| name | string | Y | 工程名称 |
| description | string | N | 描述 |
| author | string | N | 创建者 |
| created_at | ISO8601 | N | 创建时间 |
| updated_at | ISO8601 | N | 最后修改时间 |
| tags | string[] | N | 标签 |

---

## Section 2：`nodes`

**核心实体**。每个 node 代表一个物理设备/逻辑单元。其他 section（topology、scene）通过 node ID 引用。

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
        "额定功率": "150 kW",
        "工作温度": "600 ~ 850 °C",
        "燃料": "H₂ / CH₄",
        "发电效率": "≥ 60%"
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

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| label | string | Y | 显示名称 |
| label_i18n | {lang: string} | N | 多语言名称 |
| type | string | Y | 设备类型标识 (rsoc / pem / tank / compressor / fuelcell / synthesis / chp / structure / ...) |
| box | string | Y | 所属箱体 ID，对应 topology.boxes[].id |
| polemos_node_id | string | N | entelecheia PoleMos agent 的节点 ID，用于云边协同 |
| manufacturer | string | N | 厂商 |
| model | string | N | 型号 |
| serial | string | N | 序列号 |
| rated | {key: string} | N | 铭牌参数 |
| sensors | Sensor[] | N | 关联传感器 |
| status | string | N | 默认状态 (online / offline / maintenance) |
| metadata | object | N | 扩展字段 |

Sensor 结构：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | string | 传感器 ID (如 tt-101) |
| type | string | temperature / pressure / flow / level / gas / current |
| label | string | 显示标签 |
| address | string | 工业协议地址 (Modbus:HR101 / OPC-UA:ns=2;s=Temperature) |
| unit | string | 单位 |
| range | [min, max] | 量程 |

---

## Section 3：`topology`

2D 面板拓扑 — 用于 SCADA 式面板视图、2D 管线图。

```json
{
  "topology": {
    "boxes": [
      {
        "id": "box-1",
        "label": "#1 RSOC 系统",
        "label_i18n": { "zhs": "#1 RSOC 系统", "en": "#1 RSOC System" },
        "color": "#8b5cf6",
        "nodes": ["rsoc-enc", "rsoc-stack"]
      },
      {
        "id": "box-2",
        "label": "#2 电解槽区",
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
          "medium": "循环冷却水",
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

topology 字段说明：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| boxes | Box[] | 箱体分组，每个 box 包含若干 node |
| plcs | PLC[] | PLC 设备列表 |
| connections | Connections | 四类连接：信号线、电力电缆、水管、气管 |
| layout | {id: LayoutItem} | 2D 面板坐标 (每个 node / sensor / plc 的 2D 位置) |

Box 结构：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | string | 箱体 ID |
| label | string | 显示标签 |
| label_i18n | {lang: string} | 多语言 |
| color | string | 主题色 |
| nodes | string[] | 包含的 node ID 列表 |

Connection 结构（通用）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | string | 连接 ID |
| from | string | 起点实体 ID (node / sensor / plc / utility) |
| to | string | 终点实体 ID |
| points | [x,y][] | 折线路径坐标 |
| protocol | string | 信号线协议 (Modbus / 4-20mA / Profibus / HART / OPC-UA) |
| voltage | string | 电力电缆电压 |
| medium | string | 水管介质 |
| gas | string | 气管气体类型 |
| flow_rate | number | 流量 |
| temperature | number | 温度 |

---

## Section 4：`scene`

3D 全息场景配置 — 用于 webui `PhysicalPreview` 的 Three.js 渲染。

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

scene 字段说明：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| background_color | string | 3D 场景背景色 |
| environment_url | string? | HDR 环境贴图 URL |
| camera.overview | CameraView | 初始摄像机视角（正对模型全景） |
| camera.bookmarks | {boxId: CameraView} | 每个箱体的飞入目标视角 |
| lighting.ambient | {color, intensity} | 环境光 |
| lighting.directional | {color, intensity, position} | 方向光 |
| ground | {enabled, ...} | 地面配置 |
| bloom | {strength, radius, threshold} | 泛光后处理 |
| models | Model3D[] | 3D 模型放置列表 |

CameraView 结构：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| position | [x, y, z] | 摄像机位置 |
| target | [x, y, z] | 注视目标点 |
| fov | number | 视场角 (度) |

Model3D 结构：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| node | string | 关联的 node ID |
| glb | string | GLB 文件名 (相对于 models/ 目录) |
| position | [x, y, z] | 3D 世界坐标 |
| rotation | [x, y, z] | 欧拉角旋转 |
| scale | number | 缩放 |
| material | "auto" \| "holographic" \| "native" | 材质覆盖 |
| is_background | boolean | 是否为背景装饰物 |

---

## 三端对接方案

### 1. mock_scepter (Rust fixture)

加载路径：`fixtures/{project}.plant.json`

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

`mock_scepter` 启动时：

- `fixtures::load_all()` 新增 `load_plant()` 调用
- 解析 `.plant.json` → 拆分为 `DeviceModelResponse[]` + `SceneConfigItem`
- `get_scene_config` 从 plant 数据返回，不再硬编码
- `list_device_models` 从 plant 数据返回
- `topology.rs` 的 `box_detail()` / `equipment_detail()` 从 plant 数据派生

### 2. shittim-chest webui

现有 API 契约不变（`/projects/{pid}/device-models` + `/projects/{pid}/device-models/scene-config`）。

新增：

- `PhysicalPreview.tsx` 的 `BOX_CAMERA_TARGETS` 从 `scene.camera.bookmarks` 读取，不再硬编码
- 3D 模型的 CSS2D overlay 标签从 `nodes[nodeId].label` 读取

### 3. entelecheia PoleMos agent

PoleMos 通过 MCP tool 读取 plant 文件：

- `node_discover` → 遍历 `nodes` + `topology.plcs`
- `device_self_test` → 读取 `nodes[id].sensors` + `nodes[id].rated`
- 设备管理操作 → 写回 `nodes[id].status`

未来可扩展：

- PoleMos layer2 agent 生成 `.plant.json`（通过 AI 读取设备文档自动建立拓扑）
- 人在 webui 里拖拽编辑 3D 布局 → 写回 `.plant.json`
- CI/CD 管线验证 `.plant.json` 的 schema 完整性

---

## 示例文件

完整示例见 `scripts/mock/fixtures/hydrogen_corridor.plant.json`（待创建）。

---

## 与现有数据的关系

| 现有数据源 | 迁移到 .plant.json 的部分 |
| --- | --- |
| `http_server.rs` 硬编码的 20 个 DeviceModelResponse | → `scene.models[]` + `nodes{}` |
| `http_server.rs` 硬编码的 SceneConfigItem | → `scene{}` (camera, lighting, ground, bloom) |
| `mock_data/topology.rs` 的 overview() / box_detail() | → `topology{}` (boxes, connections, layout) |
| `mock_data/topology.rs` 的 equipment_detail() | → `nodes{}.rated` + `nodes{}.sensors` |
| `PhysicalPreview.tsx` 的 BOX_CAMERA_TARGETS | → `scene.camera.bookmarks` |
| `devices.json` fixture (entelecheia PoleMos) | → `nodes{}.polemos_node_id` |

## Schema 校验

JSON Schema 文件放在 `schemas/plant-v1.json`，三端共享。
`mock_scepter` 加载时 `serde_json` 反序列化 + schema 校验。
webui 构建时可用 `ajv` 校验。
entelecheia 可用 `jsonschema` crate 校验。
