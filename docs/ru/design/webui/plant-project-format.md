+++
title = "Формат файла проекта Plant (`.plant.json`)"
description = """> Дизайн формата инженерного файла — аналогично инженерному файлу Siemens TIA Portal, унифицированное описание топологии промышленных узлов, 2D-панелей, 3D-сцен."""
lang = "ru"
category = "design"
subcategory = "webui"
+++

# Формат файла проекта Plant (`.plant.json`)

> Дизайн формата инженерного файла — аналогично инженерному файлу Siemens TIA Portal, унифицированное описание топологии промышленных узлов, 2D-панелей, 3D-сцен.

## Цели дизайна

1. **Единый источник данных** — один файл описывает весь завод/проект: узлы устройств, 2D-топологию, 3D-сцену, промышленную сеть
1. **Трёхсторонняя совместимость** — `mock_scepter` (fixture), shittim-chest webui (3D-рендеринг), агент entelecheia PoleMos (управление устройствами) читают один и тот же файл
1. **Узел как центр** — вся топология, сцены, датчики привязаны к node, node является основной сущностью
1. **Версионируемость** — поле `format_version` + JSON Schema, поддержка эволюции с обратной совместимостью
1. **Расширяемость** — возможность добавления пользовательских метаданных без нарушения существующих парсеров

## Соглашения о файле

- Расширение: `.plant.json`
- Кодировка: UTF-8
- Формат: JSON (нативная поддержка на всех трёх сторонах)
- Один файл = один проект = один завод/производственная линия

## Верхнеуровневая структура

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

## Раздел 1: `metadata`

Метаданные проекта.

```json
{
  "metadata": {
    "name": "Green Hydrogen Corridor",
    "description": "Демонстрационный проект водородного коридора",
    "author": "engineering-team",
    "created_at": "2026-06-13T00:00:00Z",
    "updated_at": "2026-06-13T00:00:00Z",
    "tags": ["hydrogen", "green-energy", "demo"]
  }
}
```

Описание полей:

| Поле | Тип | Обязательное | Описание |
| --- | --- | --- | --- |
| name | string | Y | Название проекта |
| description | string | N | Описание |
| author | string | N | Создатель |
| created_at | ISO8601 | N | Время создания |
| updated_at | ISO8601 | N | Время последнего изменения |
| tags | string[] | N | Теги |

---

## Раздел 2: `nodes`

**Основная сущность**. Каждый node представляет физическое устройство/логический блок. Другие разделы (topology, scene) ссылаются по ID узла.

```json
{
  "nodes": {
    "rsoc-enc": {
      "label": "RSOC Enclosure",
      "label_i18n": { "zhs": "Корпус системы RSOC" },
      "type": "rsoc",
      "box": "box-1",
      "polemos_node_id": "node-rsoc-enc",
      "manufacturer": "Example Corp",
      "model": "RSOC-2024-ENC",
      "serial": "SN-RSOCENC-012345",
      "rated": {
        "Номинальная мощность": "150 кВт",
        "Рабочая температура": "600 ~ 850 °C",
        "Топливо": "H₂ / CH₄",
        "Эффективность генерации": "≥ 60%"
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
          "unit": "МПа",
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

Описание полей:

| Поле | Тип | Обязательное | Описание |
| --- | --- | --- | --- |
| label | string | Y | Отображаемое имя |
| label_i18n | {lang: string} | N | Многоязычное имя |
| type | string | Y | Идентификатор типа устройства (rsoc / pem / tank / compressor / fuelcell / synthesis / chp / structure / ...) |
| box | string | Y | ID корпуса, соответствует topology.boxes[].id |
| polemos_node_id | string | N | ID узла агента entelecheia PoleMos для облачно-периферийного взаимодействия |
| manufacturer | string | N | Производитель |
| model | string | N | Модель |
| serial | string | N | Серийный номер |
| rated | {key: string} | N | Паспортные параметры |
| sensors | Sensor[] | N | Связанные датчики |
| status | string | N | Статус по умолчанию (online / offline / maintenance) |
| metadata | object | N | Поле расширения |

Структура Sensor:

| Поле | Тип | Описание |
| --- | --- | --- |
| id | string | ID датчика (напр. tt-101) |
| type | string | temperature / pressure / flow / level / gas / current |
| label | string | Отображаемая метка |
| address | string | Адрес промышленного протокола (Modbus:HR101 / OPC-UA:ns=2;s=Temperature) |
| unit | string | Единица измерения |
| range | [min, max] | Диапазон |

---

## Раздел 3: `topology`

2D-топология панели — для SCADA-подобного вида панели, 2D-схемы трубопроводов.

```json
{
  "topology": {
    "boxes": [
      {
        "id": "box-1",
        "label": "#1 Система RSOC",
        "label_i18n": { "zhs": "#1 Система RSOC", "en": "#1 RSOC System" },
        "color": "#8b5cf6",
        "nodes": ["rsoc-enc", "rsoc-stack"]
      },
      {
        "id": "box-2",
        "label": "#2 Зона электролизёров",
        "label_i18n": { "zhs": "#2 Зона электролизёров", "en": "#2 Electrolyzer Area" },
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
          "medium": "Циркуляционная охлаждающая вода",
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

Описание полей topology:

| Поле | Тип | Описание |
| --- | --- | --- |
| boxes | Box[] | Группировка корпусов, каждый box содержит несколько node |
| plcs | PLC[] | Список устройств PLC |
| connections | Connections | Четыре типа соединений: сигнальные линии, силовые кабели, водопроводы, газопроводы |
| layout | {id: LayoutItem} | 2D-координаты панели (позиция 2D каждого node / sensor / plc) |

Структура Box:

| Поле | Тип | Описание |
| --- | --- | --- |
| id | string | ID корпуса |
| label | string | Отображаемая метка |
| label_i18n | {lang: string} | Многоязычность |
| color | string | Цвет темы |
| nodes | string[] | Список ID узлов |

Структура Connection (общая):

| Поле | Тип | Описание |
| --- | --- | --- |
| id | string | ID соединения |
| from | string | ID начальной сущности (node / sensor / plc / utility) |
| to | string | ID конечной сущности |
| points | [x,y][] | Координаты ломаной линии |
| protocol | string | Протокол сигнальной линии (Modbus / 4-20mA / Profibus / HART / OPC-UA) |
| voltage | string | Напряжение силового кабеля |
| medium | string | Среда водопровода |
| gas | string | Тип газа в газопроводе |
| flow_rate | number | Расход |
| temperature | number | Температура |

---

## Раздел 4: `scene`

Конфигурация 3D-голографической сцены — для рендеринга Three.js в `PhysicalPreview` webui.

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

Описание полей scene:

| Поле | Тип | Описание |
| --- | --- | --- |
| background_color | string | Цвет фона 3D-сцены |
| environment_url | string? | URL HDR-карты окружения |
| camera.overview | CameraView | Начальный вид камеры (панорама модели) |
| camera.bookmarks | {boxId: CameraView} | Целевой вид для влёта в каждый корпус |
| lighting.ambient | {color, intensity} | Окружающий свет |
| lighting.directional | {color, intensity, position} | Направленный свет |
| ground | {enabled, ...} | Конфигурация земли |
| bloom | {strength, radius, threshold} | Постобработка свечения |
| models | Model3D[] | Список размещения 3D-моделей |

Структура CameraView:

| Поле | Тип | Описание |
| --- | --- | --- |
| position | [x, y, z] | Позиция камеры |
| target | [x, y, z] | Точка обзора |
| fov | number | Угол обзора (градусы) |

Структура Model3D:

| Поле | Тип | Описание |
| --- | --- | --- |
| node | string | Связанный ID узла |
| glb | string | Имя файла GLB (относительно директории models/) |
| position | [x, y, z] | 3D-координаты в мире |
| rotation | [x, y, z] | Вращение углов Эйлера |
| scale | number | Масштаб |
| material | "auto" \| "holographic" \| "native" | Переопределение материала |
| is_background | boolean | Является ли фоновым украшением |

---

## План трёхсторонней интеграции

### 1. mock_scepter (фикстура Rust)

Путь загрузки: `fixtures/{project}.plant.json`

```text
fixtures/
├── agents.json
├── devices.json
├── hydrogen_corridor.plant.json   ← новый
└── models/
    ├── box1_rsoc_enclosure.glb
    ├── box2_alk2.glb
    └── ...
```

При запуске `mock_scepter`:

- `fixtures::load_all()` добавляет вызов `load_plant()`
- Разбор `.plant.json` → разделение на `DeviceModelResponse[]` + `SceneConfigItem`
- `get_scene_config` возвращает из данных plant, больше не жёстко закодировано
- `list_device_models` возвращает из данных plant
- `box_detail()` / `equipment_detail()` в `topology.rs` извлекаются из данных plant

### 2. shittim-chest webui

Существующий контракт API не меняется (`/projects/{pid}/device-models` + `/projects/{pid}/device-models/scene-config`).

Новое:

- `BOX_CAMERA_TARGETS` в `PhysicalPreview.tsx` читается из `scene.camera.bookmarks`, больше не жёстко закодировано
- Оверлейные метки CSS2D 3D-моделей читаются из `nodes[nodeId].label`

### 3. Агент entelecheia PoleMos

PoleMos читает файл plant через инструмент MCP:

- `node_discover` → обход `nodes` + `topology.plcs`
- `device_self_test` → чтение `nodes[id].sensors` + `nodes[id].rated`
- Операции управления устройствами → запись обратно в `nodes[id].status`

Будущие расширения:

- Агент PoleMos layer2 генерирует `.plant.json` (чтение документации устройств через AI для автоматического построения топологии)
- Человек редактирует 3D-макет перетаскиванием в webui → запись обратно в `.plant.json`
- Конвейер CI/CD проверяет целостность схемы `.plant.json`

---

## Пример файла

Полный пример см. в `scripts/mock/fixtures/hydrogen_corridor.plant.json` (будет создан).

---

## Связь с существующими данными

| Существующий источник данных | Часть, переносимая в .plant.json |
| --- | --- |
| Жёстко закодированные 20 DeviceModelResponse в `http_server.rs` | → `scene.models[]` + `nodes{}` |
| Жёстко закодированный SceneConfigItem в `http_server.rs` | → `scene{}` (camera, lighting, ground, bloom) |
| overview() / box_detail() в `mock_data/topology.rs` | → `topology{}` (boxes, connections, layout) |
| equipment_detail() в `mock_data/topology.rs` | → `nodes{}.rated` + `nodes{}.sensors` |
| BOX_CAMERA_TARGETS в `PhysicalPreview.tsx` | → `scene.camera.bookmarks` |
| Фикстура `devices.json` (entelecheia PoleMos) | → `nodes{}.polemos_node_id` |

## Проверка схемы

Файл JSON Schema находится в `schemas/plant-v1.json`, общий для трёх сторон.
`mock_scepter` при загрузке: десериализация `serde_json` + проверка схемы.
webui при сборке: можно проверять через `ajv`.
entelecheia: можно проверять через крейт `jsonschema`.
