+++
title = "Diseño del Sistema de Configuración TOML de Proveedores"
description = """El Sistema de Configuración TOML de Proveedores migra toda la configuración de Proveedores LLM de valores hardcodeados a archivos de configuración TOML, logrando la separación de configuración y código, mejorando la mantenibilidad"""
lang = "es"
category = "design"
subcategory = "core"
+++

# Diseño del Sistema de Configuración TOML de Proveedores

## Descripción General

El Sistema de Configuración TOML de Proveedores migra toda la configuración de Proveedores LLM de valores hardcodeados a archivos de configuración TOML, logrando la separación de configuración y código, mejorando la mantenibilidad y extensibilidad.

## Objetivos Principales

| Objetivo | Descripción |
| --- | --- |
| Mantenibilidad | Configuración separada del código, sin necesidad de recompilación para cambios |
| Extensibilidad | Añadir un nuevo Proveedor solo requiere añadir un archivo TOML |
| Legibilidad | Los archivos de configuración son claros y fáciles de entender |
| Reutilización | La configuración puede compartirse entre diferentes entornos |

## Diseño de Arquitectura

### Proceso de Carga de Configuración

```mermaid
flowchart TB
    subgraph Fase de Inicialización
        A[Inicio de Aplicación] --> B[Escanear Directorio res/]
        B --> C[Cargar Todos los Archivos .toml]
        C --> D[Analizar Estructura TOML]
    end

    subgraph Fase de Validación
        D --> E{Validar Completitud de Configuración}
        E -->|Pasa| F[Almacenar en Caché de Configuración]
        E -->|Falla| G[Registrar Error]
        G --> H[Usar Configuración Predeterminada]
    end

    subgraph Tiempo de Ejecución
        F --> I[Solicitud de Proveedor]
        I --> J[Obtener Configuración de Caché]
        J --> K[Devolver ProviderConfig]
    end
```

### Jerarquía de Configuración

```mermaid
graph TB
    subgraph ProviderConfig
        A[Información del Proveedor]
        B[Configuración API]
        C[Configuración de Límites]
        D[Configuración de Precios]
        E[Configuración de Capacidades]
        F[Lista de Modelos]
    end

    A --> A1[id, nombre, tipo, protocolo]
    B --> B1[url_base, endpoints, autenticación]
    C --> C1[límites de concurrencia, límites de tasa, timeout]
    D --> D1[modo de facturación, información de cuota]
    E --> E1[streaming, visión, function_calling]
    F --> F1[Lista de ModelConfig]

    subgraph ModelConfig
        F1 --> M1[id, nombre, ventana_contexto]
        F1 --> M2[flags de soporte de capacidad]
        F1 --> M3[información de precios]
        F1 --> M4[datos de benchmark]
    end
```

## Prioridad de Configuración

```mermaid
graph LR
    A[Configuración de Usuario] -->|Prioridad Máxima| D[Configuración Efectiva]
    B[Configuración de Comunidad] -->|Prioridad Media| D
    C[Configuración Oficial] -->|Prioridad Base| D

    style A fill:#90EE90
    style B fill:#FFD700
    style C fill:#87CEEB
```

### Reglas de Fusión por Prioridad

| Capa | Origen | Descripción |
| --- | --- | --- |
| 1 | Configuración Oficial | Datos de documentación oficial del proveedor, como valores predeterminados base |
| 2 | Configuración de Comunidad | Configuración optimizada contribuida por la comunidad, anula datos oficiales |
| 3 | Configuración de Usuario | Configuración definida por el usuario, máxima prioridad |

## Modelos de Precios

```mermaid
stateDiagram-v2
    [*] --> PagoPorUso: Pago Por Uso
    [*] --> PagoÚnico: Compra Única
    [*] --> Periódico: Cuota Periódica
    [*] --> Gratuito: Gratuito

    PagoPorUso --> Medir Uso
    PagoÚnico --> Verificar Saldo
    Periódico --> Verificar Cuota del Período
    Gratuito --> Ilimitado
```

### Comparación de Modelos de Precios

| Modelo | Escenarios Aplicables | Características |
| --- | --- | --- |
| PagoPorUso | OpenAI, Anthropic | Pago por token, deducción en tiempo real |
| PagoÚnico | Paquetes prepagados | Compra previa de cuota, usar hasta agotar |
| Periódico | GLM China, etc. | Reinicio de cuota periódica |
| Gratuito | Modelos locales Ollama | Sin límites de coste |

## Clasificación de Tipos de Proveedor

```mermaid
graph TB
    subgraph Proveedores en la Nube
        A[Protocolo Compatible OpenAI]
        B[Protocolo Anthropic]
        C[Protocolo Google Gemini]
    end

    subgraph Proveedores Locales
        D[Ollama]
        E[LocalAI]
    end

    subgraph Proveedores Personalizados
        F[Endpoints Definidos por Usuario]
    end

    A --> A1[OpenAI, DeepSeek, Qwen]
    B --> B1[Serie Claude]
    C --> C1[Serie Gemini]
```

## Mecanismo de Recarga en Caliente

```mermaid
sequenceDiagram
    participant FS as Sistema de Archivos
    participant Watcher as Observador de Configuración
    participant Cache as Caché de Configuración
    participant App as Aplicación

    FS->>Watcher: Evento de Cambio de Archivo
    Watcher->>Watcher: Analizar Contenido Modificado
    Watcher->>Cache: Actualizar Caché
    Cache->>App: Enviar Notificación de Actualización de Configuración
    App->>App: Aplicar Nueva Configuración
```

## Estrategia de Manejo de Errores

```mermaid
flowchart TB
    A[Carga de Configuración] --> B{¿Análisis Exitoso?}
    B -->|Sí| C[Validar Configuración]
    B -->|No| D[Registrar Error de Análisis]

    C --> E{¿Validación Exitosa?}
    E -->|Sí| F[Almacenar en Caché]
    E -->|No| G[Registrar Error de Validación]

    D --> H[Usar Configuración Predeterminada]
    G --> H

    F --> I[Uso Normal]
    H --> I
```

## Diseño de Extensibilidad

### Añadir Nuevo Proveedor

```mermaid
flowchart LR
    A[Crear Archivo TOML] --> B[Definir Información del Proveedor]
    B --> C[Configurar Endpoints API]
    C --> D[Añadir Lista de Modelos]
    D --> E[Establecer Información de Precios]
    E --> F[Reiniciar Aplicación]
    F --> G[Carga Automática de Configuración]
```

### Reglas de Validación de Configuración

| Campo | Regla de Validación | Manejo de Error |
| --- | --- | --- |
| provider.id | No vacío, único | Rechazar carga, registrar error |
| api.base_url | Formato URL válido | Usar valor predeterminado |
| models[].id | No vacío | Omitir ese modelo |
| pricing.model | Verificación de valor enum | Predeterminado PagoPorUso |

## Consideraciones de Seguridad

```mermaid
flowchart TB
    subgraph Manejo de Información Sensible
        A[Clave API] --> B[Almacenamiento Cifrado]
        B --> C[Uso en Memoria]
        C --> D[Enmascaramiento en Registros]
    end

    subgraph Control de Acceso
        E[Lectura de Configuración] --> F{Verificación de Permiso}
        F -->|Tiene Permiso| G[Devolver Configuración]
        F -->|Sin Permiso| H[Denegar Acceso]
    end
```

## Extensiones Futuras

| Característica | Descripción | Prioridad |
| --- | --- | --- |
| Recarga en Caliente de Configuración | Cargar archivos de configuración externos en tiempo de ejecución | Alta |
| Validación de Configuración | Validar completitud de configuración al inicio | Alta |
| Fusión de Configuración | Configuración de usuario anula configuración predeterminada | Media |
| Importar/Exportar Configuración | Soporte para importar/exportar archivos de configuración | Media |
| Actualización de Agente | Auto-actualizar configuración desde documentos oficiales | Baja |

# Diseño de Gestión de Metadatos de Proveedores

## Descripción General

El sistema de Gestión de Metadatos de Proveedores es responsable de obtener dinámicamente información de configuración de la documentación oficial de Proveedores LLM, permitiendo actualizaciones automatizadas y validación de datos de configuración.

## Problema Principal

La implementación actual contiene estadísticas de uso hardcodeadas y carece de soporte dinámico de datos de Proveedores. Es necesario establecer un mecanismo automatizado de adquisición y gestión de metadatos.

## Diseño de Arquitectura

### Arquitectura de Flujo de Datos

```mermaid
flowchart TB
    subgraph Fuentes de Datos
        A[Documentos Oficiales]
        B[Endpoints API]
        C[Contribuciones de la Comunidad]
    end

    subgraph Capa de Recopilación
        D[Agente de Configuración]
        E[Scraper Web]
        F[Cliente API]
    end

    subgraph Capa de Procesamiento
        G[Analizador de Datos]
        H[Motor de Validación]
        I[Estrategia de Fusión]
    end

    subgraph Capa de Almacenamiento
        J[Base de Datos de Configuración]
        K[Capa de Caché]
    end

    A --> D
    B --> F
    C --> D
    D --> G
    E --> G
    F --> G
    G --> H
    H --> I
    I --> J
    J --> K
```

### Modelo de Prioridad de Configuración

```mermaid
graph TB
    subgraph Capas de Prioridad
        A[Configuración de Usuario] -->|Máxima| D[Configuración Efectiva]
        B[Configuración de Comunidad] -->|Media| D
        C[Configuración Oficial] -->|Base| D
    end

    subgraph Reglas de Fusión
        D --> E[Anulación a Nivel de Campo]
        E --> F[Mantener Valor de Mayor Prioridad]
    end
```

## Estructura de Metadatos

### Jerarquía de Configuración del Proveedor

```mermaid
classDiagram
    class ProviderConfig {
        +provider_id: String
        +display_name: String
        +available_models: List~ModelConfig~
        +default_model: String
        +pricing_model: PricingModel
        +usage_type: UsageType
        +api_endpoint: String
    }

    class ModelConfig {
        +model_id: String
        +model_name: String
        +context_window: u64
        +max_output_tokens: u64
        +supports_vision: bool
        +supports_function_calling: bool
    }

    class PricingModel {
        <<enumeration>>
        OneTime
        Periodic
        PayAsYouGo
    }

    class UsageType {
        <<enumeration>>
        Metered
        Quota
        Unlimited
    }

    ProviderConfig --> ModelConfig
    ProviderConfig --> PricingModel
    ProviderConfig --> UsageType
```

### Clasificación de Origen de Configuración

| Tipo de Origen | Descripción | Fiabilidad | Frecuencia de Actualización |
| --- | --- | --- | --- |
| Oficial | Documentación oficial del proveedor | Alta | Periódica automática |
| Comunidad | Datos contribuidos por la comunidad | Media | Actualización manual |
| Anulación de Usuario | Personalizado por el usuario | Máxima | Tiempo real |

## Sistema de Recopilación por Agentes

### Proceso de Recopilación

```mermaid
sequenceDiagram
    participant Scheduler as Planificador
    participant Agent as Agente de Configuración
    participant Source as Fuente de Datos
    participant Parser as Analizador
    participant Validator as Validador
    participant DB as Base de Datos

    Scheduler->>Agent: Activar tarea de recopilación
    Agent->>Source: Solicitar documentos oficiales
    Source-->>Agent: Devolver HTML/JSON
    Agent->>Parser: Analizar contenido
    Parser-->>Agent: Datos estructurados
    Agent->>Validator: Validar datos
    Validator-->>Agent: Resultado de validación
    Agent->>DB: Almacenar configuración
    DB-->>Agent: Almacenamiento exitoso
    Agent-->>Scheduler: Tarea completada
```

### Responsabilidades del Agente de Proveedor

```mermaid
flowchart LR
    subgraph Agente OpenAI
        A1[Obtener lista de modelos]
        A2[Analizar información de precios]
        A3[Extraer límites de tasa]
    end

    subgraph Agente Anthropic
        B1[Obtener modelos Claude]
        B2[Analizar ventana de contexto]
        B3[Extraer información de capacidad]
    end

    subgraph Agente GLM
        C1[Obtener modelos GLM]
        C2[Analizar información de cuota]
        C3[Extraer período de reinicio]
    end
```

## Mecanismo de Validación de Datos

### Proceso de Validación

```mermaid
flowchart TB
    A[Recibir datos de configuración] --> B{Validación de formato}
    B -->|Pasa| C{Validación lógica}
    B -->|Falla| D[Registrar error]

    C -->|Pasa| E{Validación de completitud}
    C -->|Falla| D

    E -->|Pasa| F{Validación de consistencia}
    E -->|Falla| G[Rellenar valores predeterminados]

    F -->|Pasa| H[Aceptar configuración]
    F -->|Falla| I[Marcar para revisión]

    G --> F
    D --> J[Rechazar configuración]
```

### Reglas de Validación

| Tipo de Validación | Contenido Verificado | Manejo de Fallo |
| --- | --- | --- |
| Validación de formato | Tipos de datos, formatos de campo | Rechazar y registrar |
| Validación lógica | Rangos de valores, valores enum | Usar valores predeterminados |
| Validación de completitud | Campos requeridos existen | Rellenar valores predeterminados |
| Validación de consistencia | Relaciones entre campos correctas | Marcar para revisión |

## Estrategia de Fusión de Configuración

### Fusión a Nivel de Campo

```mermaid
flowchart TB
    subgraph Entrada
        A[Configuración Oficial]
        B[Configuración de Comunidad]
        C[Configuración de Usuario]
    end

    subgraph Proceso de Fusión
        D[Por prioridad de campo]
        E[Mantener valores no nulos]
        F[Validar resultado]
    end

    A --> D
    B --> D
    C --> D
    D --> E
    E --> F
    F --> G[Configuración Efectiva]
```

### Ejemplo de Fusión

| Campo | Valor Oficial | Valor Comunidad | Valor Usuario | Valor Final |
| --- | --- | --- | --- | --- |
| context_window | 128000 | - | 64000 | 64000 |
| max_concurrent | 100 | 50 | - | 50 |
| pricing_model | PagoPorUso | - | - | PagoPorUso |

## Interfaz de Configuración de Usuario

### Estructura del Archivo de Configuración

```mermaid
graph TB
    subgraph Archivo de Configuración de Usuario
        A[Nombre mostrado del proveedor]
        B[Configuración de tipo de uso]
        C[Límites de cuota]
        D[Control de concurrencia]
        E[Gestión de contexto]
        F[Anulaciones de modelo]
    end

    A --> A1[Nombre mostrado personalizado]
    B --> B1[medido/cuota/ilimitado]
    C --> C1[Límite de datos/Período de recuperación]
    D --> D1[Máximo concurrente]
    E --> E1[Límite teórico/Límite práctico]
    F --> F1[Lista de modelos personalizada]
```

## Mecanismo de Actualización Programada

```mermaid
sequenceDiagram
    participant Timer as Temporizador
    participant Queue as Cola de Tareas
    participant Agent as Grupo de Agentes
    participant DB as Base de Datos

    Timer->>Queue: Añadir tarea de actualización
    Queue->>Agent: Asignar tarea

    loop Cada Proveedor
        Agent->>Agent: Obtener última configuración
        Agent->>DB: Comparar cambios
        alt Hay cambios
            DB->>DB: Actualizar configuración
            DB->>DB: Registrar cambios
        else Sin cambios
            DB->>DB: Actualizar hora de verificación
        end
    end

    Agent-->>Queue: Tarea completada
```

## Manejo de Errores

### Manejo de Fallo de Recopilación

```mermaid
flowchart TB
    A[Recopilación fallida] --> B{Tipo de fallo}
    B -->|Error de red| C[Mecanismo de reintento]
    B -->|Error de análisis| D[Registrar y omitir]
    B -->|Error de validación| E[Marcar para revisión]

    C --> F{Contador de reintentos}
    F -->|No excedido| G[Reintento con retraso]
    F -->|Excedido| H[Usar datos en caché]

    G --> A
    D --> I[Continuar siguiente]
    E --> J[Cola de revisión manual]
```

## Diseño de Extensibilidad

### Añadir Nuevo Proveedor

```mermaid
flowchart LR
    A[Definir Agente] --> B[Implementar interfaz de recopilación]
    B --> C[Configurar reglas de análisis]
    C --> D[Registrar en planificador]
    D --> E[Iniciar recopilación]
```

### Puntos de Extensión

| Tipo de Extensión | Descripción | Implementación |
| --- | --- | --- |
| Nuevo Proveedor | Añadir nueva fuente de configuración | Implementar interfaz de Agente de Proveedor |
| Nuevo campo | Extender estructura de configuración | Actualizar modelo de datos y reglas de validación |
| Nueva regla de validación | Añadir lógica de validación | Añadir implementación de validador |

## Implementación de Agente de Capa 3

### Agente ProviderScratch

`ProviderScratch` es el primer Agente oficial de Capa 3, sirviendo como implementación de ejemplo de instalaciones de scraping.

```mermaid
flowchart TB
    subgraph Agente ProviderScratch
        A[Entrada del Agente] --> B{Modo de Ejecución}
        B -->|Modo TUI| C[Interfaz Interactiva]
        B -->|Modo CI| D[Ejecución Automatizada]

        C --> E[Seleccionar Proveedor]
        D --> F[Leer variables de entorno]

        E --> G[Llamar Habilidad]
        F --> G

        G --> H[Scrapear documentos]
        H --> I[Analizar datos]
        I --> J[Generar TOML]

        J --> K{¿Confirmar commit?}
        K -->|Sí| L[Escribir en espacio de trabajo]
        K -->|No| M[Descartar cambios]

        L --> N[Solicitar commit al usuario]
    end
```

### Arquitectura de Habilidades

Cada Proveedor corresponde a una Habilidad independiente:

```mermaid
graph LR
    subgraph Habilidades
        A[openai]
        B[anthropic]
        C[glm]
        D[deepseek]
        E[qwen]
        F[gemini]
    end

    subgraph Componentes Compartidos
        G[Scraper de Documentos]
        H[Analizador de Datos]
        I[Generador TOML]
    end

    A --> G
    B --> G
    C --> G
    D --> G
    E --> G
    F --> G

    G --> H
    H --> I
```

### Estructura de Directorios

```mermaid
flowchart LR
    Root[".amphoreus/provider_scratch/"]
    AT["agent.toml"]
    OV["overview/"]
    SK["skills/"]
    Root --> AT
    Root --> OV
    Root --> SK
    OV --> ZH["zhs.md"]
    SK --> OA["openai/"]
    SK --> AN["anthropic/"]
    SK --> GL["glm/"]
    SK --> DS["deepseek/"]
    SK --> QW["qwen/"]
    SK --> GE["gemini/"]
    OA --> OAP["prompt.md"]
    AN --> ANP["prompt.md"]
    GL --> GLP["prompt.md"]
    DS --> DSP["prompt.md"]
    QW --> QWP["prompt.md"]
    GE --> GEP["prompt.md"]
```

### Automatización CI

```mermaid
flowchart LR
    A[Disparador programado] --> B[Checkout de código]
    B --> C[Ejecutar ProviderScratch]
    C --> D{Detectar cambios}
    D -->|Hay cambios| E[Crear rama]
    E --> F[Commitear cambios]
    F --> G[Crear PR]
    G --> H[Esperar revisión]
    D -->|Sin cambios| I[Completar]
```

### Variables de Entorno

| Nombre de Variable | Descripción |
| --- | --- |
| `AMPHOREUS_PROVIDER_SCRATCH_PROVIDERS` | Lista de proveedores a scrapear |
| `AMPHOREUS_PROVIDER_SCRATCH_OUTPUT_DIR` | Ruta del directorio de salida |
| `AMPHOREUS_PROVIDER_SCRATCH_GIT_BRANCH` | Rama Git objetivo |
| `AMPHOREUS_PROVIDER_SCRATCH_DRY_RUN` | Solo simulación |

## Planes Futuros

| Característica | Descripción | Prioridad |
| --- | --- | --- |
| Control de versiones de configuración | Rastrear historial de cambios de configuración | Alta |
| Notificación de cambios | Notificar a usuarios sobre actualizaciones de configuración | Media |
| Reversión de configuración | Soporte para revertir a versiones históricas | Media |
| Recomendaciones inteligentes | Recomendar configuraciones basadas en patrones de uso | Baja |
| Agente GitHub巡回 | Auto-crear PRs para actualizar configuraciones | Alta |
