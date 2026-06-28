+++
title = "플랜트 프로젝트 파일 형식 (`.plant.json`)"
description = """> 공정 파일 형식 설계 — 지멘스 TIA Portal과 유사한 공정 파일로, 산업 노드 토폴로지, 2D 패널, 3D 장면을 통합 기술한다."""
lang = "ko"
category = "design"
subcategory = "webui"
+++

# 플랜트 프로젝트 파일 형식 (`.plant.json`)

> 공정 파일 형식 설계 — 지멘스 TIA Portal과 유사한 공정 파일로, 산업 노드 토폴로지, 2D 패널, 3D 장면을 통합 기술한다.

## 설계 목표

1. **단일 데이터 소스** — 하나의 파일이 전체 공장/프로젝트를 기술한다: 장치 노드, 2D 토폴로지, 3D 장면, 산업 네트워크
1. **3자 호환** — `mock_scepter` (fixture), shittim-chest webui (3D 렌더링), entelecheia PoleMos agent (장치 관리) 모두 동일한 파일을 읽는다
1. **노드 중심** — 모든 토폴로지, 장면, 센서가 node에 연결되며, node가 핵심 엔티티이다
1. **버전 관리 가능** — `format_version` 필드 + JSON Schema, 하위 호환 진화 지원
1. **확장 가능** — 기존 파서를 깨지 않고 사용자 정의 metadata 추가 허용

## 파일 규칙

- 확장자: `.plant.json`
- 인코딩: UTF-8
- 형식: JSON (3단 모두 네이티브 지원)
- 각 파일 = 하나의 프로젝트 = 하나의 공장/생산 라인

## 최상위 구조

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

프로젝트 메타데이터.

```json
{
  "metadata": {
    "name": "Green Hydrogen Corridor",
    "description": "수소 에너지 회랑 시범 공정",
    "author": "engineering-team",
    "created_at": "2026-06-13T00:00:00Z",
    "updated_at": "2026-06-13T00:00:00Z",
    "tags": ["hydrogen", "green-energy", "demo"]
  }
}
```

필드 설명:

| 필드 | 타입 | 필수 | 설명 |
| --- | --- | --- | --- |
| name | string | Y | 프로젝트 이름 |
| description | string | N | 설명 |
| author | string | N | 작성자 |
| created_at | ISO8601 | N | 생성 시간 |
| updated_at | ISO8601 | N | 최종 수정 시간 |
| tags | string[] | N | 태그 |

---

## Section 2: `nodes`

**핵심 엔티티**. 각 node는 하나의 물리적 장치/논리적 유닛을 나타낸다. 다른 섹션(topology, scene)은 node ID를 통해 참조한다.

```json
{
  "nodes": {
    "rsoc-enc": {
      "label": "RSOC Enclosure",
      "label_i18n": { "ko": "RSOC 시스템 외함" },
      "type": "rsoc",
      "box": "box-1",
      "polemos_node_id": "node-rsoc-enc",
      "manufacturer": "Example Corp",
      "model": "RSOC-2024-ENC",
      "serial": "SN-RSOCENC-012345",
      "rated": {
        "정격 출력": "150 kW",
        "작동 온도": "600 ~ 850 °C",
        "연료": "H₂ / CH₄",
        "발전 효율": "≥ 60%"
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

필드 설명:

| 필드 | 타입 | 필수 | 설명 |
| --- | --- | --- | --- |
| label | string | Y | 표시 이름 |
| label_i18n | {lang: string} | N | 다국어 이름 |
| type | string | Y | 장치 타입 식별자 (rsoc / pem / tank / compressor / fuelcell / synthesis / chp / structure / ...) |
| box | string | Y | 소속 함체 ID, topology.boxes[].id에 대응 |
| polemos_node_id | string | N | entelecheia PoleMos agent의 노드 ID, 클라우드-엣지 협업용 |
| manufacturer | string | N | 제조사 |
| model | string | N | 모델명 |
| serial | string | N | 시리얼 번호 |
| rated | {key: string} | N | 명판 파라미터 |
| sensors | Sensor[] | N | 연결된 센서 |
| status | string | N | 기본 상태 (online / offline / maintenance) |
| metadata | object | N | 확장 필드 |

Sensor 구조:

| 필드 | 타입 | 설명 |
| --- | --- | --- |
| id | string | 센서 ID (예: tt-101) |
| type | string | temperature / pressure / flow / level / gas / current |
| label | string | 표시 라벨 |
| address | string | 산업 프로토콜 주소 (Modbus:HR101 / OPC-UA:ns=2;s=Temperature) |
| unit | string | 단위 |
| range | [min, max] | 측정 범위 |

---

## Section 3: `topology`

2D 패널 토폴로지 — SCADA식 패널 뷰, 2D 배관도에 사용된다.

```json
{
  "topology": {
    "boxes": [
      {
        "id": "box-1",
        "label": "#1 RSOC 시스템",
        "label_i18n": { "ko": "#1 RSOC 시스템", "en": "#1 RSOC System" },
        "color": "#8b5cf6",
        "nodes": ["rsoc-enc", "rsoc-stack"]
      },
      {
        "id": "box-2",
        "label": "#2 전해조 구역",
        "label_i18n": { "ko": "#2 전해조 구역", "en": "#2 Electrolyzer Area" },
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
          "medium": "순환 냉각수",
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

topology 필드 설명:

| 필드 | 타입 | 설명 |
| --- | --- | --- |
| boxes | Box[] | 함체 그룹화, 각 box는 여러 node를 포함 |
| plcs | PLC[] | PLC 장치 목록 |
| connections | Connections | 4종 연결: 신호선, 전력 케이블, 수도관, 가스관 |
| layout | {id: LayoutItem} | 2D 패널 좌표 (각 node / sensor / plc의 2D 위치) |

Box 구조:

| 필드 | 타입 | 설명 |
| --- | --- | --- |
| id | string | 함체 ID |
| label | string | 표시 라벨 |
| label_i18n | {lang: string} | 다국어 |
| color | string | 테마 색상 |
| nodes | string[] | 포함된 node ID 목록 |

Connection 구조 (공통):

| 필드 | 타입 | 설명 |
| --- | --- | --- |
| id | string | 연결 ID |
| from | string | 시작점 엔티티 ID (node / sensor / plc / utility) |
| to | string | 종료점 엔티티 ID |
| points | [x,y][] | 폴리라인 경로 좌표 |
| protocol | string | 신호선 프로토콜 (Modbus / 4-20mA / Profibus / HART / OPC-UA) |
| voltage | string | 전력 케이블 전압 |
| medium | string | 수도관 매질 |
| gas | string | 가스관 가스 유형 |
| flow_rate | number | 유량 |
| temperature | number | 온도 |

---

## Section 4: `scene`

3D 홀로그램 장면 구성 — webui PhysicalPreview의 Three.js 렌더링에 사용된다.

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

scene 필드 설명:

| 필드 | 타입 | 설명 |
| --- | --- | --- |
| background_color | string | 3D 장면 배경색 |
| environment_url | string? | HDR 환경 맵 URL |
| camera.overview | CameraView | 초기 카메라 시점 (모델 파노라마 정면) |
| camera.bookmarks | {boxId: CameraView} | 각 함체의 플라이인 목표 시점 |
| lighting.ambient | {color, intensity} | 환경광 |
| lighting.directional | {color, intensity, position} | 방향광 |
| ground | {enabled, ...} | 지면 구성 |
| bloom | {strength, radius, threshold} | 블룸 후처리 |
| models | Model3D[] | 3D 모델 배치 목록 |

CameraView 구조:

| 필드 | 타입 | 설명 |
| --- | --- | --- |
| position | [x, y, z] | 카메라 위치 |
| target | [x, y, z] | 주시 목표점 |
| fov | number | 시야각 (도) |

Model3D 구조:

| 필드 | 타입 | 설명 |
| --- | --- | --- |
| node | string | 연결된 node ID |
| glb | string | GLB 파일명 (models/ 디렉터리 기준) |
| position | [x, y, z] | 3D 월드 좌표 |
| rotation | [x, y, z] | 오일러 각 회전 |
| scale | number | 스케일 |
| material | "auto" \| "holographic" \| "native" | 재질 오버라이드 |
| is_background | boolean | 배경 장식 여부 |

---

## 3단 연동 방안

### 1. mock_scepter (Rust fixture)

로드 경로: `fixtures/{project}.plant.json`

```text
fixtures/
├── agents.json
├── devices.json
├── hydrogen_corridor.plant.json   ← 신규
└── models/
    ├── box1_rsoc_enclosure.glb
    ├── box2_alk2.glb
    └── ...
```

`mock_scepter` 시작 시:

- `fixtures::load_all()`에 `load_plant()` 호출 추가
- `.plant.json` 파싱 → `DeviceModelResponse[]` + `SceneConfigItem`으로 분할
- `get_scene_config`가 plant 데이터에서 반환, 하드코딩 제거
- `list_device_models`가 plant 데이터에서 반환
- `topology.rs`의 `box_detail()` / `equipment_detail()`이 plant 데이터에서 파생

### 2. shittim-chest webui

기존 API 계약 불변 (`/projects/{pid}/device-models` + `/projects/{pid}/device-models/scene-config`).

신규:

- `PhysicalPreview.tsx`의 `BOX_CAMERA_TARGETS`가 `scene.camera.bookmarks`에서 읽기, 하드코딩 제거
- 3D 모델의 CSS2D 오버레이 라벨이 `nodes[nodeId].label`에서 읽기

### 3. entelecheia PoleMos agent

PoleMos가 MCP tool을 통해 plant 파일 읽기:

- `node_discover` → `nodes` + `topology.plcs` 순회
- `device_self_test` → `nodes[id].sensors` + `nodes[id].rated` 읽기
- 장치 관리 작업 → `nodes[id].status`에 쓰기

향후 확장 가능:

- PoleMos layer2 agent가 `.plant.json` 생성 (AI가 장치 문서를 읽어 자동 토폴로지 구축)
- 사용자가 webui에서 3D 레이아웃 드래그 편집 → `.plant.json`에 쓰기
- CI/CD 파이프라인이 `.plant.json`의 스키마 완전성 검증

---

## 예제 파일

전체 예제는 `scripts/mock/fixtures/hydrogen_corridor.plant.json` (생성 예정) 참조.

---

## 기존 데이터와의 관계

| 기존 데이터 소스 | .plant.json으로 마이그레이션되는 부분 |
| --- | --- |
| `http_server.rs`에 하드코딩된 20개 DeviceModelResponse | → `scene.models[]` + `nodes{}` |
| `http_server.rs`에 하드코딩된 SceneConfigItem | → `scene{}` (camera, lighting, ground, bloom) |
| `mock_data/topology.rs`의 overview() / box_detail() | → `topology{}` (boxes, connections, layout) |
| `mock_data/topology.rs`의 equipment_detail() | → `nodes{}.rated` + `nodes{}.sensors` |
| `PhysicalPreview.tsx`의 BOX_CAMERA_TARGETS | → `scene.camera.bookmarks` |
| `devices.json` fixture (entelecheia PoleMos) | → `nodes{}.polemos_node_id` |

## 스키마 검증

JSON Schema 파일은 `schemas/plant-v1.json`에 위치하며, 3단이 공유한다.
`mock_scepter` 로드 시 `serde_json` 역직렬화 + 스키마 검증.
webui 빌드 시 `ajv`로 검증 가능.
entelecheia는 `jsonschema` crate으로 검증 가능.
