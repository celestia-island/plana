+++
title = "Conception de l'Ordonnancement de Conteneurs et du Routage de Jetons Cosmos"
description = """Ce document décrit l'architecture d'ordonnancement des conteneurs Cosmos : comment les outils MCP marqués avec `ToolLocation::Cosmos` sont routés via JSON-RPC sur socket Unix vers leurs conteneurs"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# Conception de l'Ordonnancement de Conteneurs et du Routage de Jetons Cosmos

## Aperçu

Ce document décrit l'architecture d'ordonnancement des conteneurs Cosmos : comment les outils MCP marqués avec `ToolLocation::Cosmos` sont routés via JSON-RPC sur socket Unix vers leurs conteneurs correspondants, et comment le système de jetons (numéro d'agent) s'articule avec l'identité et le routage des conteneurs.

## I. Modèle de Localisation des Outils

### Double Environnement d'Exécution

```mermaid
flowchart LR
    subgraph Scepter["Scepter (Processus Central)"]
        A1[Appels LLM]
        A2[Requêtes RAG]
        A3[Gestion des Tâches]
        A4[Stockage des Credentials]
    end

    subgraph Cosmos["Cosmos (Conteneur Par Agent)"]
        P1[Accès au Système de Fichiers]
        P2[Exécution de Scripts]
        P3[Accès Matériel]
        P4[Sessions REPL]
    end

    Scepter -->|ToolLocation::Scepter| Local[Invocation Locale]
    Cosmos -->|ToolLocation::Cosmos| Socket[RPC Socket Unix]
```

### Énumération ToolLocation

| Variante | Site d'Exécution | Transport |
| --- | --- | --- |
| `Scepter` (par défaut) | En processus via `McpToolInvoker` | Appel de fonction direct |
| `Cosmos` | En conteneur via `CosmosConnector` | JSON-RPC socket Unix |

### Critères de Décision de Localisation

```mermaid
flowchart TD
    Outil[Outil MCP] --> Q1{Besoin de Ressources Conteneur ?}
    Q1 -->|Oui : système de fichiers, scripts, matériel| Cosmos[ToolLocation::Cosmos]
    Q1 -->|Non : LLM, RAG, gestion d'état| Scepter[ToolLocation::Scepter]
```

Les outils qui nécessitent des ressources de conteneur (système de fichiers, exécution de scripts, accès matériel) sont marqués `Cosmos`. Les services centralisés (LLM, RAG, gestion des tâches, interaction humaine) restent `Scepter`.

## II. Système de Jetons et Identité de Conteneur

### Allocation de Numéro d'Agent

```mermaid
sequenceDiagram
    participant SM as Gestionnaire SkillChain
    participant AIM as AgentIdManager
    participant SC as SnowflakeContainer
    participant PC as CosmosConnector

    SM->>AIM: Demander un numéro d'agent
    AIM-->>SM: Assigner le jeton 000-999
    SM->>SC: Créer un conteneur pour le jeton
    SC-->>SM: UUID du conteneur + chemin du socket
    SM->>PC: connect(UUID, socket_path)
    PC-->>SM: Connexion établie
```

### Propriétés du Jeton

| Propriété | Description |
| --- | --- |
| Format | Nombre à trois chiffres : `000`-`999` |
| Allocateur | `AgentIdManager` dans la chaîne de compétences |
| Liaison | Un jeton par panneau de chaîne de compétences |
| Affichage | Affiché dans la ligne de statistiques TUI comme `cosmos#NNN` |
| Persistance | Survit aux redémarrages d'agent |

## III. Flux de Routage des Requêtes

### Appel MCP Initié par TUI

```mermaid
sequenceDiagram
    participant TUI as Client TUI
    participant MSR as mcp_skill_router
    participant AM as AgentManager
    participant BI as BridgeInvoker
    participant BS as HapLotesBridgeServer
    participant PR as McpRouter (Cosmos)

    TUI->>MSR: McpMessage::CallTool(tool_name, agent_type, params)
    MSR->>AM: get_tool_location(tool_name)
    AM-->>MSR: ToolLocation

    alt ToolLocation::Cosmos
        MSR->>AM: invoke_tool(tool_name, agent_type, params)
        AM->>BI: router vers le bon agent
        BI->>BS: transmettre via le pont HapLotes
        BS->>PR: bridge.call(tool_name, params)
        PR-->>BS: résultat
        BS-->>BI: réponse JSON-RPC
        BI-->>AM: McpToolResult
        AM-->>MSR: McpToolResult
        MSR-->>TUI: McpMessage::ToolResponse
    else ToolLocation::Scepter
        MSR->>AM: router via la passerelle HapLotes
        AM->>AM: mcp_tools.invoke() localement
        AM-->>TUI: McpMessage::ToolResponse via WS
    end
```

### Logique de Routage Clé

La décision de routage se produit dans `mcp_skill_router.rs` :

1. Vérifier `agent_manager.get_tool_location(tool_name)`
1. Si `ToolLocation::Cosmos` et mode conteneurisé actif :

   - Appeler `agent_manager.invoke_tool()` qui route via `BridgeInvoker` → pont HapLotes → `McpRouter` de Cosmos
   - Le `McpRouter` de Cosmos distribue localement (skemma) ou retourne à Scepter via le pont pour les agents distants
   - Retourner `McpMessage::ToolResponse` directement à TUI

1. Sinon : router via la passerelle HapLotes vers le processus de l'agent

## IV. Architecture CosmosConnector / Pont

### Pont HapLotes (Actuel)

Le pont HapLotes est le **seul canal de communication** entre Scepter et les conteneurs Cosmos.

```mermaid
flowchart LR
    subgraph Cosmos["Cosmos (Conteneur)"]
        MR[McpRouter] -->|ToolSource::Local| SK[skemma Boa JS]
        MR -->|ToolSource::Bridge| BC[HapLotesBridgeClient]
    end

    subgraph Scepter["Scepter (Hôte)"]
        BS[HapLotesBridgeServer] --> BI[BridgeInvoker]
        BI --> AG1[Aporia]
        BI --> AG2[KaLos]
        BI --> AG3[...tous les agents]
    end

    BC -->|Socket Unix JSON-RPC| BS
```

### Pool de Connexions (CosmosConnector — côté Scepter)

```mermaid
classDiagram
    class CosmosConnector {
        -connections: RwLock~HashMap~String, CosmosConnection~~
        +connect(instance_uuid, socket_path) Result
        +invoke_tool(instance_uuid, tool_name, params) Result~Value~
        +list_tools(instance_uuid) Result~Vec~String~~
        +disconnect(instance_uuid)
    }

    class CosmosConnection {
        -transport: Mutex~JsonRpcTransport~
    }

    class JsonRpcTransport {
        +send_request(request) Result~JsonRpcResponse~
    }

    CosmosConnector --> CosmosConnection
    CosmosConnection --> JsonRpcTransport
```

### Protocole JSON-RPC

Tous les noms de méthode utilisent l'énumération `UnixMethod` pour la sécurité de type à la compilation :

| Variante UnixMethod | Direction | Paramètres |
| --- | --- | --- |
| `UnixMethod::McpCall` | Scepter → Cosmos | `{ tool_name, parameters }` |
| `UnixMethod::McpListTools` | Scepter → Cosmos | Aucun |
| `UnixMethod::ReplSnapshot` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::ReplRestore` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::BridgeCall` | Cosmos → Scepter | `{ tool_name, parameters }` |
| `UnixMethod::BridgeListTools` | Cosmos → Scepter | Aucun |

### Format de Réponse

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## V. Cycle de Vie du Conteneur

```mermaid
stateDiagram-v2
    [*] --> EnAttente: Chaîne de Compétences Démarrée
    EnAttente --> Création: SnowflakeManager.create()
    Création --> Démarrage: Conteneur en Cours d'Exécution
    Démarrage --> Connecté: CosmosConnector.connect()
    Connecté --> Prêt: Vérification de Santé Réussie

    Prêt --> Exécution: invoke_tool()
    Exécution --> Prêt: Résultat Retourné

    Prêt --> Arrêt: Chaîne de Compétences Terminée
    Arrêt --> Déconnecté: CosmosConnector.disconnect()
    Déconnecté --> [*]

    Création --> Échec: Délai d'Attente
    Démarrage --> Échec: Erreur de Connexion
    Échec --> [*]
```

### Agents de Conteneur

À l'intérieur des conteneurs Cosmos, seul skemma s'exécute localement (moteur Boa JS). Tous les autres outils d'agent passent par le pont HapLotes vers Scepter :

| Agent | Rôle | Dans Cosmos ? |
| --- | --- | --- |
| SkeMma | Exécution de scripts (Boa JS) | **Local** (en processus) |
| Aporia | Chat LLM | Via pont → Scepter |
| KaLos | E/S Fichier | Via pont → Scepter |
| NeiKos | Gestion de conteneurs | Via pont → Scepter |
| EleOs | Recherche web | Via pont → Scepter |
| Tous les autres | Divers | Via pont → Scepter |

## VI. Intégration de la Ligne de Statistiques

### Format d'Affichage

Dans la TUI `AgentDetailPage`, la ligne de statistiques affiche :

```mermaid
flowchart LR
    BORDER["|"] --> TOK["1,2k jetons"] --> SEP1["|"] --> DUR["3,5s"] --> SEP2["|"] --> COSMOS["cosmos#042"] --> TIER["[T2]"]

    TOK -.->|"McpToolResult.token_usage"| SRC1["Utilisation de Jetons"]
    DUR -.->|"Instant::now()"| SRC2["Durée"]
    COSMOS -.->|"AgentIdManager"| SRC3["Numéro d'Agent"]
    TIER -.->|"McpToolConfig.tier"| SRC4["Niveau de Modèle"]
```

| Segment | Source |
| --- | --- |
| `1,2k jetons` | `McpToolResult.token_usage` |
| `3,5s` | Durée depuis `Instant::now()` |
| `cosmos#042` | Numéro d'agent depuis `AgentIdManager` |
| `[T2]` | Niveau de modèle depuis `McpToolConfig.tier` |

## VII. Gestion des Erreurs

### Modes de Défaillance

```mermaid
flowchart TD
    Appel[Appel d'Outil] --> Q1{Conteneur en Ligne ?}
    Q1 -->|Non| E1[Erreur : AGENT_HORS_LIGNE]
    Q1 -->|Oui| Q2{Socket Connecté ?}
    Q2 -->|Non| E2[Erreur : Connexion Perdue]
    Q2 -->|Oui| Q3{Outil Existe ?}
    Q3 -->|Non| E3[Erreur : Outil Non Trouvé]
    Q3 -->|Oui| Q4{Exécution Réussie ?}
    Q4 -->|Non| E4[Erreur : Échec d'Exécution]
    Q4 -->|Oui| Résultat[Retourner le Résultat]

    E1 --> Repli[Repli : Essayer l'exécution Scepter]
    E2 --> Réessayer[Réessayer : Reconnecter le socket]
```

### Dégradation Gracieuse

Lorsque le conteneur est indisponible, le système peut optionnellement revenir à l'exécution locale `Scepter` si l'outil a une implémentation locale enregistrée.

## VIII. Extensions Futures

| Fonctionnalité | Description | Priorité |
| --- | --- | --- |
| Pool de conteneurs | Réutiliser les conteneurs entre les chaînes de compétences | Moyenne |
| Surveillance de santé | Vérifications de santé périodiques des conteneurs | Haute |
| Limites de ressources | Limites CPU/mémoire par conteneur | Haute |
| Outils multi-conteneurs | Outils couvrant plusieurs conteneurs | Basse |
| Migration de conteneurs | Déplacer les conteneurs en cours d'exécution entre hôtes | Basse |
