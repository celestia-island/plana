+++
title = "Formato de Archivo de Proyecto de Planta (`.plant.json`)"
description = """> Diseño de formato de archivo de ingeniería — similar a los archivos de proyecto de Siemens TIA Portal, describiendo de manera unificada la topología de nodos industriales, paneles 2D y escenas 3D."""
lang = "es"
category = "design"
subcategory = "webui"
+++

# Formato de Archivo de Proyecto de Planta (`.plant.json`)

> Diseño de formato de archivo de ingeniería — similar a los archivos de proyecto de Siemens TIA Portal, describiendo de manera unificada la topología de nodos industriales, paneles 2D y escenas 3D.

## Objetivos de Diseño

1. **Fuente de datos única** — un archivo describe toda la fábrica/proyecto: nodos de dispositivos, topología 2D, escena 3D, red industrial
1. **Compatibilidad triple** — `mock_scepter` (fixture), shittim-chest webui (renderizado 3D), agente PoleMos de entelecheia (gestión de dispositivos) leen el mismo archivo
1. **Centrado en nodos** — toda la topología, escenas y sensores se vinculan a un nodo, el nodo es la entidad central
1. **Versionable** — campo `format_version` + JSON Schema, soporta evolución con compatibilidad hacia atrás
1. **Extensible** — permite añadir metadatos personalizados sin romper los analizadores existentes

## Convenciones del Archivo

- Extensión: `.plant.json`
- Codificación: UTF-8
- Formato: JSON (soporte nativo en los tres extremos)
- Cada archivo = un proyecto = una fábrica/línea de producción

## Estructura de Nivel Superior

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

## Sección 1: `metadata`

Metadatos del proyecto.

```json
{
  "metadata": {
    "name": "Corredor de Hidrógeno Verde",
    "description": "Proyecto de demostración del corredor de hidrógeno",
    "author": "equipo-ingenieria",
    "created_at": "2026-06-13T00:00:00Z",
    "updated_at": "2026-06-13T00:00:00Z",
    "tags": ["hidrogeno", "energia-verde", "demo"]
  }
}
```

Descripción de campos:

| Campo | Tipo | Obligatorio | Descripción |
| --- | --- | --- | --- |
| name | string | S | Nombre del proyecto |
| description | string | N | Descripción |
| author | string | N | Creador |
| created_at | ISO8601 | N | Fecha de creación |
| updated_at | ISO8601 | N | Fecha de última modificación |
| tags | string[] | N | Etiquetas |

---

## Sección 2: `nodes`

**Entidad central**. Cada nodo representa un dispositivo físico/unidad lógica. Otras secciones (topology, scene) hacen referencia mediante el ID del nodo.

```json
{
  "nodes": {
    "rsoc-enc": {
      "label": "RSOC Enclosure",
      "label_i18n": { "zhs": "Carcasa del sistema RSOC" },
      "type": "rsoc",
      "box": "box-1",
      "polemos_node_id": "node-rsoc-enc",
      "manufacturer": "Example Corp",
      "model": "RSOC-2024-ENC",
      "serial": "SN-RSOCENC-012345",
      "rated": {
        "Potencia nominal": "150 kW",
        "Temperatura de trabajo": "600 ~ 850 °C",
        "Combustible": "H₂ / CH₄",
        "Eficiencia de generación": "≥ 60%"
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

Descripción de campos:

| Campo | Tipo | Obligatorio | Descripción |
| --- | --- | --- | --- |
| label | string | S | Nombre mostrado |
| label_i18n | {lang: string} | N | Nombre multilingüe |
| type | string | S | Identificador de tipo de dispositivo (rsoc / pem / tank / compressor / fuelcell / synthesis / chp / structure / ...) |
| box | string | S | ID del gabinete al que pertenece, corresponde a topology.boxes[].id |
| polemos_node_id | string | N | ID del nodo del agente PoleMos de entelecheia, para colaboración nube-borde |
| manufacturer | string | N | Fabricante |
| model | string | N | Modelo |
| serial | string | N | Número de serie |
| rated | {key: string} | N | Parámetros de placa de características |
| sensors | Sensor[] | N | Sensores asociados |
| status | string | N | Estado predeterminado (online / offline / maintenance) |
| metadata | object | N | Campo de extensión |

Estructura de Sensor:

| Campo | Tipo | Descripción |
| --- | --- | --- |
| id | string | ID del sensor (ej. tt-101) |
| type | string | temperature / pressure / flow / level / gas / current |
| label | string | Etiqueta mostrada |
| address | string | Dirección de protocolo industrial (Modbus:HR101 / OPC-UA:ns=2;s=Temperature) |
| unit | string | Unidad |
| range | [min, max] | Rango |

---

## Sección 3: `topology`

Topología del panel 2D — para vistas de panel estilo SCADA, diagramas de tuberías 2D.

```json
{
  "topology": {
    "boxes": [
      {
        "id": "box-1",
        "label": "#1 Sistema RSOC",
        "label_i18n": { "zhs": "#1 Sistema RSOC", "en": "#1 RSOC System" },
        "color": "#8b5cf6",
        "nodes": ["rsoc-enc", "rsoc-stack"]
      },
      {
        "id": "box-2",
        "label": "#2 Área de electrolizadores",
        "label_i18n": { "zhs": "#2 Área de electrolizadores", "en": "#2 Electrolyzer Area" },
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
          "medium": "Agua de refrigeración en circulación",
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

Descripción de campos de topology:

| Campo | Tipo | Descripción |
| --- | --- | --- |
| boxes | Box[] | Agrupaciones de gabinetes, cada box contiene varios nodos |
| plcs | PLC[] | Lista de dispositivos PLC |
| connections | Connections | Cuatro tipos de conexiones: cables de señal, cables de potencia, tuberías de agua, tuberías de gas |
| layout | {id: LayoutItem} | Coordenadas del panel 2D (posición 2D de cada nodo / sensor / plc) |

Estructura de Box:

| Campo | Tipo | Descripción |
| --- | --- | --- |
| id | string | ID del gabinete |
| label | string | Etiqueta mostrada |
| label_i18n | {lang: string} | Multilingüe |
| color | string | Color del tema |
| nodes | string[] | Lista de IDs de nodos contenidos |

Estructura de Connection (genérica):

| Campo | Tipo | Descripción |
| --- | --- | --- |
| id | string | ID de conexión |
| from | string | ID de entidad origen (nodo / sensor / plc / utilidad) |
| to | string | ID de entidad destino |
| points | [x,y][] | Coordenadas de la ruta poligonal |
| protocol | string | Protocolo de cable de señal (Modbus / 4-20mA / Profibus / HART / OPC-UA) |
| voltage | string | Voltaje del cable de potencia |
| medium | string | Medio de la tubería de agua |
| gas | string | Tipo de gas de la tubería de gas |
| flow_rate | number | Caudal |
| temperature | number | Temperatura |

---

## Sección 4: `scene`

Configuración de escena holográfica 3D — para el renderizado Three.js de `PhysicalPreview` del webui.

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

Descripción de campos de scene:

| Campo | Tipo | Descripción |
| --- | --- | --- |
| background_color | string | Color de fondo de la escena 3D |
| environment_url | string? | URL del mapa de entorno HDR |
| camera.overview | CameraView | Vista inicial de la cámara (frente al panorama del modelo) |
| camera.bookmarks | {boxId: CameraView} | Vista objetivo de vuelo para cada gabinete |
| lighting.ambient | {color, intensity} | Luz ambiental |
| lighting.directional | {color, intensity, position} | Luz direccional |
| ground | {enabled, ...} | Configuración del suelo |
| bloom | {strength, radius, threshold} | Post-procesado de brillo |
| models | Model3D[] | Lista de colocación de modelos 3D |

Estructura de CameraView:

| Campo | Tipo | Descripción |
| --- | --- | --- |
| position | [x, y, z] | Posición de la cámara |
| target | [x, y, z] | Punto objetivo de mirada |
| fov | number | Campo de visión (grados) |

Estructura de Model3D:

| Campo | Tipo | Descripción |
| --- | --- | --- |
| node | string | ID del nodo asociado |
| glb | string | Nombre del archivo GLB (relativo al directorio models/) |
| position | [x, y, z] | Coordenadas del mundo 3D |
| rotation | [x, y, z] | Rotación de ángulos de Euler |
| scale | number | Escala |
| material | "auto" \| "holographic" \| "native" | Anulación de material |
| is_background | boolean | Si es un elemento decorativo de fondo |

---

## Plan de Integración Triple

### 1. mock_scepter (fixture Rust)

Ruta de carga: `fixtures/{project}.plant.json`

```text
fixtures/
├── agents.json
├── devices.json
├── hydrogen_corridor.plant.json   ← nuevo
└── models/
    ├── box1_rsoc_enclosure.glb
    ├── box2_alk2.glb
    └── ...
```

Al iniciar `mock_scepter`:

- `fixtures::load_all()` añade la llamada `load_plant()`
- Parsea `.plant.json` → divide en `DeviceModelResponse[]` + `SceneConfigItem`
- `get_scene_config` devuelve desde datos plant, ya no hardcodeado
- `list_device_models` devuelve desde datos plant
- `box_detail()` / `equipment_detail()` de `topology.rs` derivan de datos plant

### 2. shittim-chest webui

El contrato API existente no cambia (`/projects/{pid}/device-models` + `/projects/{pid}/device-models/scene-config`).

Nuevo:

- `PhysicalPreview.tsx` lee `BOX_CAMERA_TARGETS` desde `scene.camera.bookmarks`, ya no hardcodeado
- Las etiquetas CSS2D overlay de modelos 3D se leen desde `nodes[nodeId].label`

### 3. Agente PoleMos de entelecheia

PoleMos lee archivos plant mediante herramienta MCP:

- `node_discover` → recorre `nodes` + `topology.plcs`
- `device_self_test` → lee `nodes[id].sensors` + `nodes[id].rated`
- Operaciones de gestión de dispositivos → escribe de vuelta `nodes[id].status`

Futuro ampliable:

- El agente layer2 PoleMos genera `.plant.json` (mediante IA leyendo documentación de dispositivos para establecer topología automáticamente)
- Persona edita diseño 3D arrastrando en webui → escribe de vuelta `.plant.json`
- Pipeline CI/CD valida la integridad del schema de `.plant.json`

---

## Archivo de Ejemplo

El ejemplo completo está en `scripts/mock/fixtures/hydrogen_corridor.plant.json` (por crear).

---

## Relación con Datos Existentes

| Fuente de datos existente | Parte migrada a .plant.json |
| --- | --- |
| 20 DeviceModelResponse hardcodeados en `http_server.rs` | → `scene.models[]` + `nodes{}` |
| SceneConfigItem hardcodeado en `http_server.rs` | → `scene{}` (camera, lighting, ground, bloom) |
| overview() / box_detail() de `mock_data/topology.rs` | → `topology{}` (boxes, connections, layout) |
| equipment_detail() de `mock_data/topology.rs` | → `nodes{}.rated` + `nodes{}.sensors` |
| BOX_CAMERA_TARGETS de `PhysicalPreview.tsx` | → `scene.camera.bookmarks` |
| Fixture `devices.json` (PoleMos de entelecheia) | → `nodes{}.polemos_node_id` |

## Validación de Schema

El archivo JSON Schema se coloca en `schemas/plant-v1.json`, compartido por los tres extremos.
`mock_scepter` al cargar: deserialización `serde_json` + validación de schema.
webui en construcción: se puede validar con `ajv`.
entelecheia: se puede validar con la crate `jsonschema`.
