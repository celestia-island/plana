+++
title = "ADR-003: PostgreSQL + PgVector para Almacenamiento de Datos Unificado"
description = """Fecha: 2026-02"""
lang = "es"
category = "design"
subcategory = "core"
+++

# ADR-003: PostgreSQL + PgVector para Almacenamiento de Datos Unificado

**Fecha**: 2026-02
**Estado**: Aceptado

## Contexto

Entelecheia requiere una capa de almacenamiento de datos que sirva **dos responsabilidades distintas simultáneamente**:

1. **Datos relacionales tradicionales**: Cuentas de usuario, credenciales API, sesiones de agente, metadatos de tareas, registros de auditoría, políticas RBAC, estado de contenedores, eventos de línea de tiempo — todos datos estructurados que se benefician de integridad relacional, transacciones y consultas SQL.

1. **Búsqueda de similitud vectorial**: Embeddings para RAG (Generación Aumentada por Recuperación), sedimentación de memoria, recorrido de grafos de conocimiento y recuperación de documentos — datos que requieren búsqueda de vecinos más cercanos en alta dimensionalidad.

Se evaluaron varios enfoques de almacenamiento:

| Enfoque | Datos Relacionales | Búsqueda Vectorial | Doble Responsabilidad | Madurez |
| --- | --- | --- | --- | --- |
| **PostgreSQL + PgVector** | SQL completo, transacciones ACID | Índices HNSW/IVFFlat | Base de datos única, lenguaje de consulta único | PostgreSQL: 35+ años; PgVector: estable, ampliamente desplegado |
| **Qdrant** | Ninguno (solo vectorial) | Excelente (diseñado específicamente) | Requiere BD relacional separada | Moderada |
| **Milvus** | Ninguno (solo vectorial) | Excelente | Requiere BD relacional separada | Moderada |
| **Weaviate** | Limitado (CRUD integrado) | Bueno | Comprometido en ambos | Moderada |
| **SQLite + extensión vectorial** | SQL completo | Experimental | Archivo único, concurrencia limitada | Baja (extensión vectorial inmadura) |
| **MongoDB + Atlas Vector Search** | Almacén de documentos, sin SQL | Bueno | Modelo de consulta comprometido | Alta (pero búsqueda vectorial propietaria) |

## Decisión

Elegimos **PostgreSQL con la extensión PgVector** como backend de almacenamiento unificado.

**Razones principales:**

1. **Estabilidad y fiabilidad probada.** PostgreSQL es el motor de base de datos relacional de código abierto más probado en batalla, con 35+ años de desarrollo, un optimizador de consultas maduro, transacciones ACID sólidas como roca y un vasto ecosistema de herramientas. Para un sistema que maneja credenciales de autenticación, políticas RBAC y registros de auditoría, esta estabilidad no es opcional — es un requisito básico. Elegir una base de datos más nueva y menos probada para estas cargas de trabajo introduciría riesgo operacional innecesario.

1. **Almacenamiento unificado de doble propósito.** PgVector extiende PostgreSQL con búsqueda de similitud vectorial (indexación HNSW e IVFFlat) sin requerir una base de datos separada. Esto significa:

   - **Una base de datos que gestionar**, un pool de conexiones, una estrategia de respaldo, un sistema de migración.
   - **Consultas JOIN a través de datos relacionales y vectoriales** — ej., "encontrar documentos similares a este embedding que el usuario tiene permiso para acceder" puede expresarse como una sola consulta SQL.
   - **Consistencia transaccional** entre actualizaciones de metadatos e inserciones de embeddings.
   - **SQL como lenguaje de consulta universal** — sin necesidad de que los desarrolladores aprendan un DSL de consulta vectorial separado.

1. **PgVector es relativamente estable para nuestra escala.** Aunque no está tan optimizado como las bases de datos vectoriales diseñadas específicamente para despliegues de escala de miles de millones, PgVector maneja la carga de trabajo de Entelecheia (memorias de agentes, documentos de conocimiento, contextos RAG) competentemente. Las dimensiones de embedding (768-3072) y los tamaños de conjunto de datos (miles a millones bajos de vectores) están bien dentro de la zona de confort de PgVector.

1. **Familiaridad del equipo y ecosistema.** PostgreSQL es el motor de base de datos más ampliamente desplegado en el mundo. El equipo tiene profunda familiaridad con SQL, administración de PostgreSQL y el ecosistema ORM de Rust (SeaORM, SQLx). Elegir una base de datos vectorial desconocida requeriría una inversión de aprendizaje significativa para un beneficio marginal.

1. **Sin compromiso en compatibilidad SQL.** Muchas bases de datos vectoriales más nuevas no soportan SQL en absoluto o soportan un dialecto limitado. Esto obligaría a la aplicación a mantener dos modelos de consulta separados — uno para datos relacionales y otro para búsqueda vectorial — aumentando la complejidad del código y la superficie de errores.

## Consecuencias

### Positivas

- **Superficie operacional única**: Una base de datos que monitorear, respaldar, actualizar y depurar. Las migraciones SeaORM manejan tanto la configuración de esquema como de extensión.
- **Operaciones vectoriales transaccionales**: Las inserciones de embeddings y actualizaciones de metadatos ocurren en la misma transacción, evitando datos huérfanos o inconsistentes.
- **Potencia SQL completa para consultas combinadas**: Búsqueda vectorial consciente de permisos, similitud filtrada por tiempo y joins multi-tabla son operaciones SQL nativas.
- **Integración SeaORM + PgVector**: El ecosistema Rust tiene soporte maduro para PostgreSQL. Las entidades SeaORM pueden incluir columnas vectoriales con operadores de distancia.
- **Endurecido para producción**: La replicación de PostgreSQL, recuperación a punto en el tiempo, pooling de conexiones (PgBouncer) y monitoreo (`pg_stat_statements`) son estándar de la industria.

### Negativas

- **Techo de rendimiento de búsqueda vectorial**: Para colecciones de embeddings muy grandes (100M+ vectores), las bases de datos vectoriales diseñadas específicamente (Qdrant, Milvus) superan significativamente a PgVector. La escala actual de Entelecheia no se acerca a este límite, pero es una consideración futura.
- **PgVector es una extensión, no núcleo**: PgVector debe instalarse y mantenerse separadamente de PostgreSQL. Las imágenes de contenedor deben incluir la extensión (usamos `pgvector/pgvector:pg18`). Actualizar PostgreSQL puede requerir recompilación de la extensión.
- **Tipos de índice vectorial limitados**: PgVector soporta HNSW e IVFFlat. Las bases de datos vectoriales diseñadas específicamente ofrecen tipos de índice más especializados (ej., DiskANN, ScaNN) que pueden ser más eficientes para distribuciones específicas.
- **Competencia de recursos**: La indexación vectorial (especialmente la construcción HNSW) consume CPU y memoria que se comparte con cargas de trabajo OLTP en la misma instancia PostgreSQL. A escala, puede ser necesario separar las cargas de trabajo vectoriales a una réplica dedicada.

### Compromiso Aceptado

**Techo de rendimiento por simplicidad operacional.** Una arquitectura de doble base de datos (PostgreSQL para relacional + Qdrant/Milvus para vectores) proporcionaría mejor rendimiento de búsqueda vectorial a escala. Sin embargo, duplicaría la complejidad operacional, requeriría sincronización de datos entre dos sistemas e introduciría desafíos de consistencia. Para la escala actual y a corto plazo de Entelecheia (despliegues de usuario único a equipos pequeños), el enfoque unificado de PostgreSQL es el compromiso correcto. Si la búsqueda vectorial se convierte en un cuello de botella en el futuro, se puede introducir una réplica de lectura con PgVector o una capa de caché vectorial dedicada incrementalmente sin cambiar el modelo de consulta de la aplicación.
