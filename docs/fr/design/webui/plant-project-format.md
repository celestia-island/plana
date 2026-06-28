+++
title = "Format de Fichier de Projet d'Usine (`.plant.json`)"
description = """> Conception du format de fichier d'ingénierie — similaire aux fichiers de projet Siemens TIA Portal, décrivant de manière unifiée la topologie des nœuds industriels, les panneaux 2D et les scènes 3D."""
lang = "fr"
category = "design"
subcategory = "webui"
+++

# Format de Fichier de Projet d'Usine (`.plant.json`)

> Conception du format de fichier d'ingénierie — similaire aux fichiers de projet Siemens TIA Portal, décrivant de manière unifiée la topologie des nœuds industriels, les panneaux 2D et les scènes 3D.

## Objectifs de Conception

1. **Source de données unique** — un fichier décrit l'ensemble de l'usine/du projet : nœuds d'équipement, topologie 2D, scène 3D, réseau industriel
1. **Compatibilité tripartite** — `mock_scepter` (fixture), shittim-chest webui (rendu 3D), agent entelecheia PoleMos (gestion de périphériques) lisent tous le même fichier
1. **Centré sur les nœuds** — toute la topologie, les scènes, les capteurs sont rattachés au nœud, le nœud est l'entité centrale
1. **Versionnable** — champ `format_version` + Schéma JSON, prend en charge l'évolution rétrocompatible
1. **Extensible** — permet l'ajout de métadonnées personnalisées sans briser les analyseurs existants

## Conventions de Fichier

- Extension : `.plant.json`
- Encodage : UTF-8
- Format : JSON (support natif sur les trois plateformes)
- Chaque fichier = un projet = une usine/ligne de production

## Structure de Niveau Supérieur

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

## Section 1 : `metadata`

Métadonnées du projet.

```json
{
  "metadata": {
    "name": "Green Hydrogen Corridor",
    "description": "Projet de démonstration du corridor hydrogène",
    "author": "engineering-team",
    "created_at": "2026-06-13T00:00:00Z",
    "updated_at": "2026-06-13T00:00:00Z",
    "tags": ["hydrogen", "green-energy", "demo"]
  }
}
```

Description des champs :

| Champ | Type | Requis | Description |
| --- | --- | --- | --- |
| name | string | O | Nom du projet |
| description | string | N | Description |
| author | string | N | Créateur |
| created_at | ISO8601 | N | Date de création |
| updated_at | ISO8601 | N | Dernière modification |
| tags | string[] | N | Étiquettes |

---

## Section 2 : `nodes`

**Entité centrale**. Chaque nœud représente un équipement physique/une unité logique. Les autres sections (topology, scene) référencent par ID de nœud.

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
        "Puissance nominale": "150 kW",
        "Température de fonctionnement": "600 ~ 850 °C",
        "Combustible": "H₂ / CH₄",
        "Efficacité électrique": "≥ 60%"
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

Description des champs :

| Champ | Type | Requis | Description |
| --- | --- | --- | --- |
| label | string | O | Nom d'affichage |
| label_i18n | {lang: string} | N | Nom multilingue |
| type | string | O | Identifiant de type d'équipement (rsoc / pem / tank / compressor / fuelcell / synthesis / chp / structure / ...) |
| box | string | O | ID du boîtier d'appartenance, correspond à topology.boxes[].id |
| polemos_node_id | string | N | ID de nœud de l'agent entelecheia PoleMos, pour la collaboration cloud-edge |
| manufacturer | string | N | Fabricant |
| model | string | N | Modèle |
| serial | string | N | Numéro de série |
| rated | {key: string} | N | Paramètres de plaque signalétique |
| sensors | Sensor[] | N | Capteurs associés |
| status | string | N | État par défaut (online / offline / maintenance) |
| metadata | object | N | Champ d'extension |

Structure Sensor :

| Champ | Type | Description |
| --- | --- | --- |
| id | string | ID du capteur (ex. tt-101) |
| type | string | temperature / pressure / flow / level / gas / current |
| label | string | Étiquette d'affichage |
| address | string | Adresse de protocole industriel (Modbus:HR101 / OPC-UA:ns=2;s=Temperature) |
| unit | string | Unité |
| range | [min, max] | Plage de mesure |

---

## Section 3 : `topology`

Topologie du panneau 2D — pour la vue de panneau de type SCADA, les diagrammes de tuyauterie 2D.

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
          "medium": "Eau de refroidissement en circulation",
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

Description des champs de topology :

| Champ | Type | Description |
| --- | --- | --- |
| boxes | Box[] | Groupement par boîtiers, chaque boîte contient plusieurs nœuds |
| plcs | PLC[] | Liste des équipements PLC |
| connections | Connections | Quatre types de connexions : câbles de signal, câbles d'alimentation, conduites d'eau, conduites de gaz |
| layout | {id: LayoutItem} | Coordonnées du panneau 2D (position 2D de chaque nœud / capteur / plc) |

Structure Box :

| Champ | Type | Description |
| --- | --- | --- |
| id | string | ID du boîtier |
| label | string | Étiquette d'affichage |
| label_i18n | {lang: string} | Multilingue |
| color | string | Couleur de thème |
| nodes | string[] | Liste des ID de nœuds contenus |

Structure Connection (générique) :

| Champ | Type | Description |
| --- | --- | --- |
| id | string | ID de connexion |
| from | string | ID de l'entité source (nœud / capteur / plc / utilitaire) |
| to | string | ID de l'entité destination |
| points | [x,y][] | Coordonnées du chemin en polyligne |
| protocol | string | Protocole de câble de signal (Modbus / 4-20mA / Profibus / HART / OPC-UA) |
| voltage | string | Tension du câble d'alimentation |
| medium | string | Milieu de la conduite d'eau |
| gas | string | Type de gaz de la conduite de gaz |
| flow_rate | number | Débit |
| temperature | number | Température |

---

## Section 4 : `scene`

Configuration de scène holographique 3D — pour le rendu Three.js du `PhysicalPreview` de la webui.

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

Description des champs de scene :

| Champ | Type | Description |
| --- | --- | --- |
| background_color | string | Couleur de fond de la scène 3D |
| environment_url | string? | URL de la carte d'environnement HDR |
| camera.overview | CameraView | Vue initiale de la caméra (face au panorama du modèle) |
| camera.bookmarks | {boxId: CameraView} | Angle de vue cible pour chaque boîtier |
| lighting.ambient | {color, intensity} | Lumière ambiante |
| lighting.directional | {color, intensity, position} | Lumière directionnelle |
| ground | {enabled, ...} | Configuration du sol |
| bloom | {strength, radius, threshold} | Post-traitement de lueur |
| models | Model3D[] | Liste de placement des modèles 3D |

Structure CameraView :

| Champ | Type | Description |
| --- | --- | --- |
| position | [x, y, z] | Position de la caméra |
| target | [x, y, z] | Point cible du regard |
| fov | number | Champ de vision (degrés) |

Structure Model3D :

| Champ | Type | Description |
| --- | --- | --- |
| node | string | ID du nœud associé |
| glb | string | Nom du fichier GLB (relatif au répertoire models/) |
| position | [x, y, z] | Coordonnées mondiales 3D |
| rotation | [x, y, z] | Rotation en angles d'Euler |
| scale | number | Échelle |
| material | "auto" \| "holographic" \| "native" | Remplacement de matériau |
| is_background | boolean | S'il s'agit d'un objet décoratif d'arrière-plan |

---

## Plan d'Intégration Tripartite

### 1. mock_scepter (fixture Rust)

Chemin de chargement : `fixtures/{project}.plant.json`

```text
fixtures/
├── agents.json
├── devices.json
├── hydrogen_corridor.plant.json   ← nouveau
└── models/
    ├── box1_rsoc_enclosure.glb
    ├── box2_alk2.glb
    └── ...
```

Au démarrage de `mock_scepter` :

- `fixtures::load_all()` ajoute un appel `load_plant()`
- Analyse `.plant.json` → décomposé en `DeviceModelResponse[]` + `SceneConfigItem`
- `get_scene_config` retourne depuis les données plant, plus codé en dur
- `list_device_models` retourne depuis les données plant
- `box_detail()` / `equipment_detail()` de `topology.rs` dérivent des données plant

### 2. shittim-chest webui

Le contrat API existant est inchangé (`/projects/{pid}/device-models` + `/projects/{pid}/device-models/scene-config`).

Nouveautés :

- `PhysicalPreview.tsx` `BOX_CAMERA_TARGETS` lit depuis `scene.camera.bookmarks`, plus codé en dur
- Les étiquettes CSS2D overlay des modèles 3D lisent depuis `nodes[nodeId].label`

### 3. Agent entelecheia PoleMos

PoleMos lit les fichiers plant via l'outil MCP :

- `node_discover` → parcourt `nodes` + `topology.plcs`
- `device_self_test` → lit `nodes[id].sensors` + `nodes[id].rated`
- Opérations de gestion de périphériques → écrit `nodes[id].status`

Extensions futures :

- L'agent PoleMos layer2 génère `.plant.json` (l'IA lit la documentation de l'équipement et établit automatiquement la topologie)
- Édition par glisser-déposer de la disposition 3D dans la webui → écriture dans `.plant.json`
- Pipeline CI/CD valide l'intégrité du schéma `.plant.json`

---

## Fichier d'Exemple

L'exemple complet se trouve dans `scripts/mock/fixtures/hydrogen_corridor.plant.json` (à créer).

---

## Relation avec les Données Existantes

| Source de données existante | Partie migrée vers .plant.json |
| --- | --- |
| 20 DeviceModelResponse codés en dur dans `http_server.rs` | → `scene.models[]` + `nodes{}` |
| SceneConfigItem codé en dur dans `http_server.rs` | → `scene{}` (camera, lighting, ground, bloom) |
| overview() / box_detail() de `mock_data/topology.rs` | → `topology{}` (boxes, connections, layout) |
| equipment_detail() de `mock_data/topology.rs` | → `nodes{}.rated` + `nodes{}.sensors` |
| BOX_CAMERA_TARGETS de `PhysicalPreview.tsx` | → `scene.camera.bookmarks` |
| fixture `devices.json` (entelecheia PoleMos) | → `nodes{}.polemos_node_id` |

## Validation de Schéma

Le fichier JSON Schema est placé dans `schemas/plant-v1.json`, partagé entre les trois plateformes.
`mock_scepter` au chargement : désérialisation `serde_json` + validation de schéma.
La webui peut utiliser `ajv` pour la validation lors de la construction.
entelecheia peut utiliser la crate `jsonschema` pour la validation.
