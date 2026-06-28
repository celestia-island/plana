+++
title = "Diseño de Programación de Contenedores Cosmos y Enrutamiento de Tokens"
description = """Este documento describe la arquitectura de programación de contenedores Cosmos: cómo las herramientas MCP marcadas con `ToolLocation::Cosmos` se enrutan a través de JSON-RPC por Unix socket a sus cont"""
lang = "es"
category = "design"
subcategory = "core"
+++

# Diseño de Programación de Contenedores Cosmos y Enrutamiento de Tokens

## Descripción General

Este documento describe la arquitectura de programación de contenedores Cosmos: cómo las herramientas MCP marcadas con `ToolLocation::Cosmos` se enrutan a través de JSON-RPC por Unix socket a sus contenedores correspondientes, y cómo el sistema de tokens (número de agente) se vincula con la identidad y el enrutamiento del contenedor.

## I. Modelo de Ubicación de Herramientas

### Entorno de Ejecución Dual

```mermaid
flowchart LR
    subgraph Scepter["Scepter (Proceso Central)"]
        A1[Llamadas LLM]
        A2[Consultas RAG]
        A3[Gestión de Tareas]
        A4[Almacenamiento de Credenciales]
    end

    subgraph Cosmos["Cosmos (Contenedor Por Agente)"]
        P1[Acceso al Sistema de Archivos]
        P2[Ejecución de Scripts]
        P3[Acceso a Hardware]
        P4[Sesiones REPL]
    end

    Scepter -->|ToolLocation::Scepter| Local[Invocación Local]
    Cosmos -->|ToolLocation::Cosmos| Socket[RPC por Unix Socket]
```

### Enum ToolLocation

| Variante | Sitio de Ejecución | Transporte |
| --- | --- | --- |
| `Scepter` (predeterminado) | En proceso mediante `McpToolInvoker` | Llamada directa a función |
| `Cosmos` | En contenedor mediante `CosmosConnector` | JSON-RPC por Unix socket |

### Criterios de Decisión de Ubicación

```mermaid
flowchart TD
    Tool[Herramienta MCP] --> Q1{¿Necesita Recursos de Contenedor?}
    Q1 -->|Sí: sistema de archivos, scripts, hardware| Cosmos[ToolLocation::Cosmos]
    Q1 -->|No: LLM, RAG, gestión de estado| Scepter[ToolLocation::Scepter]
```

Las herramientas que requieren recursos de contenedor (sistema de archivos, ejecución de scripts, acceso a hardware) se marcan `Cosmos`. Los servicios centralizados (LLM, RAG, gestión de tareas, interacción humana) permanecen `Scepter`.

## II. Sistema de Tokens e Identidad del Contenedor

### Asignación de Número de Agente

```mermaid
sequenceDiagram
    participant SM as Gestor de Cadena de Habilidades
    participant AIM as AgentIdManager
    participant SC as SnowflakeContainer
    participant PC as CosmosConnector

    SM->>AIM: Solicitar número de agente
    AIM-->>SM: Asignar token 000-999
    SM->>SC: Crear contenedor para token
    SC-->>SM: UUID de contenedor + ruta de socket
    SM->>PC: connect(UUID, socket_path)
    PC-->>SM: Conexión establecida
```

### Propiedades del Token

| Propiedad | Descripción |
| --- | --- |
| Formato | Número de tres dígitos: `000`-`999` |
| Asignador | `AgentIdManager` en la cadena de habilidades |
| Vinculación | Un token por panel de cadena de habilidades |
| Visualización | Mostrado en la línea de estadísticas TUI como `cosmos#NNN` |
| Persistencia | Sobrevive a reinicios de agente |

## III. Flujo de Enrutamiento de Solicitudes

### Llamada MCP Originada en TUI

```mermaid
sequenceDiagram
    participant TUI as Cliente TUI
    participant MSR as mcp_skill_router
    participant AM as AgentManager
    participant BI as BridgeInvoker
    participant BS as HapLotesBridgeServer
    participant PR as McpRouter (Cosmos)

    TUI->>MSR: McpMessage::CallTool(nombre_herramienta, tipo_agente, params)
    MSR->>AM: get_tool_location(nombre_herramienta)
    AM-->>MSR: ToolLocation

    alt ToolLocation::Cosmos
        MSR->>AM: invoke_tool(nombre_herramienta, tipo_agente, params)
        AM->>BI: enrutar al agente correcto
        BI->>BS: reenviar mediante puente HapLotes
        BS->>PR: bridge.call(nombre_herramienta, params)
        PR-->>BS: resultado
        BS-->>BI: respuesta JSON-RPC
        BI-->>AM: McpToolResult
        AM-->>MSR: McpToolResult
        MSR-->>TUI: McpMessage::ToolResponse
    else ToolLocation::Scepter
        MSR->>AM: enrutar a través de puerta de enlace HapLotes
        AM->>AM: mcp_tools.invoke() localmente
        AM-->>TUI: McpMessage::ToolResponse vía WS
    end
```

### Lógica de Enrutamiento Clave

La decisión de enrutamiento ocurre en `mcp_skill_router.rs`:

1. Verificar `agent_manager.get_tool_location(nombre_herramienta)`
1. Si `ToolLocation::Cosmos` y modo contenedorizado activo:

   - Llamar `agent_manager.invoke_tool()` que enruta a través de `BridgeInvoker` → puente HapLotes → `McpRouter` de Cosmos
   - El `McpRouter` de Cosmos despacha localmente (skemma) o de vuelta a Scepter mediante puente para agentes remotos
   - Devolver `McpMessage::ToolResponse` directamente a TUI

1. De lo contrario: enrutar a través de la puerta de enlace HapLotes al proceso del agente

## IV. Arquitectura CosmosConnector / Puente

### Puente HapLotes (Actual)

El puente HapLotes es el **único canal de comunicación** entre Scepter y los contenedores Cosmos.

```mermaid
flowchart LR
    subgraph Cosmos["Cosmos (Contenedor)"]
        MR[McpRouter] -->|ToolSource::Local| SK[Boa JS de skemma]
        MR -->|ToolSource::Bridge| BC[HapLotesBridgeClient]
    end

    subgraph Scepter["Scepter (Host)"]
        BS[HapLotesBridgeServer] --> BI[BridgeInvoker]
        BI --> AG1[Aporia]
        BI --> AG2[KaLos]
        BI --> AG3[...todos los agentes]
    end

    BC -->|JSON-RPC por Unix Socket| BS
```

### Pool de Conexiones (CosmosConnector — lado Scepter)

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

### Protocolo JSON-RPC

Todos los nombres de método usan el enum `UnixMethod` para seguridad de tipos en tiempo de compilación:

| Variante UnixMethod | Dirección | Parámetros |
| --- | --- | --- |
| `UnixMethod::McpCall` | Scepter → Cosmos | `{ tool_name, parameters }` |
| `UnixMethod::McpListTools` | Scepter → Cosmos | Ninguno |
| `UnixMethod::ReplSnapshot` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::ReplRestore` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::BridgeCall` | Cosmos → Scepter | `{ tool_name, parameters }` |
| `UnixMethod::BridgeListTools` | Cosmos → Scepter | Ninguno |

### Formato de Respuesta

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## V. Ciclo de Vida del Contenedor

```mermaid
stateDiagram-v2
    [*] --> Pending: Cadena de Habilidades Iniciada
    Pending --> Creating: SnowflakeManager.create()
    Creating --> Starting: Contenedor Ejecutándose
    Starting --> Connected: CosmosConnector.connect()
    Connected --> Ready: Verificación de Salud Superada

    Ready --> Executing: invoke_tool()
    Executing --> Ready: Resultado Devuelto

    Ready --> Stopping: Cadena de Habilidades Completa
    Stopping --> Disconnected: CosmosConnector.disconnect()
    Disconnected --> [*]

    Creating --> Failed: Tiempo de Espera
    Starting --> Failed: Error de Conexión
    Failed --> [*]
```

### Agentes de Contenedor

Dentro de los contenedores Cosmos, solo skemma se ejecuta localmente (motor Boa JS). Todas las demás herramientas de agente se enrutan a través del puente HapLotes de vuelta a Scepter:

| Agente | Rol | ¿En Cosmos? |
| --- | --- | --- |
| SkeMma | Ejecución de scripts (Boa JS) | **Local** (en proceso) |
| Aporia | Chat LLM | Vía puente → Scepter |
| KaLos | E/S de archivos | Vía puente → Scepter |
| NeiKos | Gestión de contenedores | Vía puente → Scepter |
| EleOs | Búsqueda web | Vía puente → Scepter |
| Todos los demás | Varios | Vía puente → Scepter |

## VI. Integración de Línea de Estadísticas

### Formato de Visualización

En la `AgentDetailPage` de TUI, la línea de estadísticas muestra:

```mermaid
flowchart LR
    BORDER["|"] --> TOK["1.2k tokens"] --> SEP1["|"] --> DUR["3.5s"] --> SEP2["|"] --> COSMOS["cosmos#042"] --> TIER["[T2]"]

    TOK -.->|"McpToolResult.token_usage"| SRC1["Uso de Tokens"]
    DUR -.->|"Instant::now()"| SRC2["Duración"]
    COSMOS -.->|"AgentIdManager"| SRC3["Número de Agente"]
    TIER -.->|"McpToolConfig.tier"| SRC4["Nivel de Modelo"]
```

| Segmento | Origen |
| --- | --- |
| `1.2k tokens` | `McpToolResult.token_usage` |
| `3.5s` | Duración desde `Instant::now()` |
| `cosmos#042` | Número de agente de `AgentIdManager` |
| `[T2]` | Nivel de modelo de `McpToolConfig.tier` |

## VII. Manejo de Errores

### Modos de Fallo

```mermaid
flowchart TD
    Call[Llamada a Herramienta] --> Q1{¿Contenedor en Línea?}
    Q1 -->|No| E1[Error: AGENT_OFFLINE]
    Q1 -->|Sí| Q2{¿Socket Conectado?}
    Q2 -->|No| E2[Error: Conexión Perdida]
    Q2 -->|Sí| Q3{¿Herramienta Existe?}
    Q3 -->|No| E3[Error: Herramienta No Encontrada]
    Q3 -->|Sí| Q4{¿Ejecución Exitosa?}
    Q4 -->|No| E4[Error: Ejecución Fallida]
    Q4 -->|Sí| Result[Devolver Resultado]

    E1 --> Fallback[Respaldo: Intentar ejecución Scepter]
    E2 --> Retry[Reintentar: Reconectar socket]
```

### Degradación con Gracia

Cuando el contenedor no está disponible, el sistema puede opcionalmente recurrir a la ejecución local `Scepter` si la herramienta tiene una implementación local registrada.

## VIII. Extensiones Futuras

| Característica | Descripción | Prioridad |
| --- | --- | --- |
| Pooling de contenedores | Reutilizar contenedores entre cadenas de habilidades | Media |
| Monitoreo de salud | Verificaciones periódicas de salud del contenedor | Alta |
| Límites de recursos | Límites de CPU/memoria por contenedor | Alta |
| Herramientas multi-contenedor | Herramientas que abarcan múltiples contenedores | Baja |
| Migración de contenedores | Mover contenedores en ejecución entre hosts | Baja |
