+++
title = "プラントプロジェクトファイル形式 (`.plant.json`)"
description = """> 工程ファイル形式設計 — シーメンスのTIA Portalに類似した工程ファイルで、工業ノードトポロジー、2Dパネル、3Dシーンを統一的に記述します。"""
lang = "ja"
category = "design"
subcategory = "webui"
+++

# プラントプロジェクトファイル形式 (`.plant.json`)

> 工程ファイル形式設計 — シーメンスのTIA Portalに類似した工程ファイルで、工業ノードトポロジー、2Dパネル、3Dシーンを統一的に記述します。

## 設計目標

1. **単一データソース** — 1つのファイルで工場/プロジェクト全体を記述：デバイスノード、2Dトポロジー、3Dシーン、工業ネットワーク
1. **三者互換** — `mock_scepter`（フィクスチャ）、shittim-chest webui（3Dレンダリング）、entelecheia PoleMosエージェント（デバイス管理）のすべてが同一ファイルを読み取ります
1. **ノード中心** — すべてのトポロジー、シーン、センサーはnodeに紐付けられ、nodeが核心エンティティです
1. **バージョン管理可能** — `format_version`フィールド + JSON Schema、後方互換性のある進化をサポート
1. **拡張可能** — 既存のパーサーを破壊せずにカスタムメタデータの追加を許可

## ファイル規約

- 拡張子：`.plant.json`
- エンコーディング：UTF-8
- 形式：JSON（三者すべてがネイティブサポート）
- 各ファイル = 1プロジェクト = 1工場/生産ライン

## トップレベル構造

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

## セクション1: `metadata`

工程メタデータ。

```json
{
  "metadata": {
    "name": "Green Hydrogen Corridor",
    "description": "水素エネルギー回廊実証プロジェクト",
    "author": "engineering-team",
    "created_at": "2026-06-13T00:00:00Z",
    "updated_at": "2026-06-13T00:00:00Z",
    "tags": ["hydrogen", "green-energy", "demo"]
  }
}
```

フィールド説明：

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| name | string | Y | プロジェクト名 |
| description | string | N | 説明 |
| author | string | N | 作成者 |
| created_at | ISO8601 | N | 作成日時 |
| updated_at | ISO8601 | N | 最終更新日時 |
| tags | string[] | N | タグ |

---

## セクション2: `nodes`

**核心エンティティ**。各nodeは物理デバイス/論理ユニットを表します。他のセクション（topology、scene）はnode IDで参照します。

```json
{
  "nodes": {
    "rsoc-enc": {
      "label": "RSOC Enclosure",
      "label_i18n": { "zhs": "RSOC システム筐体" },
      "type": "rsoc",
      "box": "box-1",
      "polemos_node_id": "node-rsoc-enc",
      "manufacturer": "Example Corp",
      "model": "RSOC-2024-ENC",
      "serial": "SN-RSOCENC-012345",
      "rated": {
        "定格出力": "150 kW",
        "動作温度": "600 ~ 850 °C",
        "燃料": "H₂ / CH₄",
        "発電効率": "≥ 60%"
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

フィールド説明：

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| label | string | Y | 表示名 |
| label_i18n | {lang: string} | N | 多言語名 |
| type | string | Y | デバイスタイプ識別子 (rsoc / pem / tank / compressor / fuelcell / synthesis / chp / structure / ...) |
| box | string | Y | 所属筐体ID、topology.boxes[].idに対応 |
| polemos_node_id | string | N | entelecheia PoleMosエージェントのノードID、クラウドエッジ連携用 |
| manufacturer | string | N | メーカー |
| model | string | N | 型番 |
| serial | string | N | シリアル番号 |
| rated | {key: string} | N | 銘板パラメータ |
| sensors | Sensor[] | N | 関連センサー |
| status | string | N | デフォルト状態 (online / offline / maintenance) |
| metadata | object | N | 拡張フィールド |

Sensor構造：

| フィールド | 型 | 説明 |
| --- | --- | --- |
| id | string | センサーID (例: tt-101) |
| type | string | temperature / pressure / flow / level / gas / current |
| label | string | 表示ラベル |
| address | string | 工業プロトコルアドレス (Modbus:HR101 / OPC-UA:ns=2;s=Temperature) |
| unit | string | 単位 |
| range | [min, max] | レンジ |

---

## セクション3: `topology`

2Dパネルトポロジー — SCADA形式のパネルビュー、2D配管図に使用。

```json
{
  "topology": {
    "boxes": [
      {
        "id": "box-1",
        "label": "#1 RSOC システム",
        "label_i18n": { "zhs": "#1 RSOC システム", "en": "#1 RSOC System" },
        "color": "#8b5cf6",
        "nodes": ["rsoc-enc", "rsoc-stack"]
      },
      {
        "id": "box-2",
        "label": "#2 電解槽エリア",
        "label_i18n": { "zhs": "#2 電解槽エリア", "en": "#2 Electrolyzer Area" },
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
          "medium": "循環冷却水",
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

topologyフィールド説明：

| フィールド | 型 | 説明 |
| --- | --- | --- |
| boxes | Box[] | 筐体グループ、各boxは複数のnodeを含む |
| plcs | PLC[] | PLCデバイスリスト |
| connections | Connections | 4種類の接続：信号線、電力ケーブル、水管、ガス管 |
| layout | {id: LayoutItem} | 2Dパネル座標（各node / sensor / plcの2D位置） |

Box構造：

| フィールド | 型 | 説明 |
| --- | --- | --- |
| id | string | 筐体ID |
| label | string | 表示ラベル |
| label_i18n | {lang: string} | 多言語 |
| color | string | テーマカラー |
| nodes | string[] | 含まれるnode IDリスト |

Connection構造（共通）：

| フィールド | 型 | 説明 |
| --- | --- | --- |
| id | string | 接続ID |
| from | string | 起点エンティティID (node / sensor / plc / utility) |
| to | string | 終点エンティティID |
| points | [x,y][] | 折れ線パス座標 |
| protocol | string | 信号線プロトコル (Modbus / 4-20mA / Profibus / HART / OPC-UA) |
| voltage | string | 電力ケーブル電圧 |
| medium | string | 水管媒体 |
| gas | string | ガス管ガスタイプ |
| flow_rate | number | 流量 |
| temperature | number | 温度 |

---

## セクション4: `scene`

3Dホログラフィックシーン設定 — webui PhysicalPreviewのThree.jsレンダリング用。

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

sceneフィールド説明：

| フィールド | 型 | 説明 |
| --- | --- | --- |
| background_color | string | 3Dシーン背景色 |
| environment_url | string? | HDR環境マップURL |
| camera.overview | CameraView | 初期カメラ視点（モデル全景を正対） |
| camera.bookmarks | {boxId: CameraView} | 各筐体へのフライインターゲット視点 |
| lighting.ambient | {color, intensity} | 環境光 |
| lighting.directional | {color, intensity, position} | ディレクショナルライト |
| ground | {enabled, ...} | 地面設定 |
| bloom | {strength, radius, threshold} | ブルームポストプロセス |
| models | Model3D[] | 3Dモデル配置リスト |

CameraView構造：

| フィールド | 型 | 説明 |
| --- | --- | --- |
| position | [x, y, z] | カメラ位置 |
| target | [x, y, z] | 注視ターゲット点 |
| fov | number | 視野角（度） |

Model3D構造：

| フィールド | 型 | 説明 |
| --- | --- | --- |
| node | string | 関連node ID |
| glb | string | GLBファイル名（models/ディレクトリからの相対パス） |
| position | [x, y, z] | 3Dワールド座標 |
| rotation | [x, y, z] | オイラー角回転 |
| scale | number | スケール |
| material | "auto" \| "holographic" \| "native" | マテリアルオーバーライド |
| is_background | boolean | 背景装飾物かどうか |

---

## 三者連携方案

### 1. mock_scepter（Rustフィクスチャ）

ロードパス：`fixtures/{project}.plant.json`

```text
fixtures/
├── agents.json
├── devices.json
├── hydrogen_corridor.plant.json   ← 新規追加
└── models/
    ├── box1_rsoc_enclosure.glb
    ├── box2_alk2.glb
    └── ...
```

mock_scepter起動時：

- `fixtures::load_all()`に`load_plant()`呼び出しを新規追加
- `.plant.json`を解析 → `DeviceModelResponse[]` + `SceneConfigItem`に分割
- `get_scene_config`はplantデータから返し、ハードコードを廃止
- `list_device_models`はplantデータから返す
- `topology.rs`の`box_detail()` / `equipment_detail()`はplantデータから派生

### 2. shittim-chest webui

既存API契約は変更なし（`/projects/{pid}/device-models` + `/projects/{pid}/device-models/scene-config`）。

新規追加：

- `PhysicalPreview.tsx`の`BOX_CAMERA_TARGETS`を`scene.camera.bookmarks`から読み取り、ハードコードを廃止
- 3DモデルのCSS2Dオーバーレイラベルを`nodes[nodeId].label`から読み取り

### 3. entelecheia PoleMosエージェント

PoleMosはMCPツール経由でplantファイルを読み取ります：

- `node_discover` → `nodes` + `topology.plcs`を走査
- `device_self_test` → `nodes[id].sensors` + `nodes[id].rated`を読み取り
- デバイス管理操作 → `nodes[id].status`に書き戻し

将来の拡張：

- PoleMos layer2エージェントが`.plant.json`を生成（AIがデバイスドキュメントを読み取り自動的にトポロジーを構築）
- ユーザーがwebuiで3Dレイアウトをドラッグ編集 → `.plant.json`に書き戻し
- CI/CDパイプラインで`.plant.json`のスキーマ完全性を検証

---

## サンプルファイル

完全なサンプルは`scripts/mock/fixtures/hydrogen_corridor.plant.json`を参照（作成予定）。

---

## 既存データとの関係

| 既存データソース | .plant.jsonに移行する部分 |
| --- | --- |
| `http_server.rs`にハードコードされた20個のDeviceModelResponse | → `scene.models[]` + `nodes{}` |
| `http_server.rs`にハードコードされたSceneConfigItem | → `scene{}` (camera, lighting, ground, bloom) |
| `mock_data/topology.rs`のoverview() / box_detail() | → `topology{}` (boxes, connections, layout) |
| `mock_data/topology.rs`のequipment_detail() | → `nodes{}.rated` + `nodes{}.sensors` |
| `PhysicalPreview.tsx`のBOX_CAMERA_TARGETS | → `scene.camera.bookmarks` |
| `devices.json`フィクスチャ（entelecheia PoleMos） | → `nodes{}.polemos_node_id` |

## スキーマ検証

JSON Schemaファイルは`schemas/plant-v1.json`に配置され、三者共有。
mock_scepterロード時に`serde_json`逆シリアル化 + スキーマ検証。
webuiビルド時に`ajv`で検証可能。
entelecheiaは`jsonschema`クレートで検証可能。
