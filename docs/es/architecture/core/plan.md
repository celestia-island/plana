+++
title = "Plan de Integración del Corredor Industrial"
description = """> Objetivo: El sistema debe demostrar auto-interconexión autónoma con un corredor de demostración industrial"""
lang = "es"
category = "architecture"
subcategory = "core"
+++

# Plan de Integración Autónoma del Corredor Industrial

> **Objetivo**: El sistema debe demostrar **auto-interconexión autónoma** con un
> corredor de demostración industrial completamente desconocido — descubriendo hardware,
> infiriendo modelos de datos, generando configuración de monitorización y cerrando el
> bucle alarma→respuesta — sin ingeniería manual por dispositivo.
> **Fecha límite gubernamental**: esta capacidad está vinculada a un hito de proyecto gubernamental.

---

## Trabajo Restante

La cadena completa de descubrimiento → inferencia → monitorización → alarma →
**aprobación de escritura** está entregada (Fases A.1–A.3, B, C, D.1, **D.2 ✓**).
El único trabajo restante es la **validación dogfood de extremo a extremo (Fase E)**
— operacional, no código.

### D.2 — Ciclo completo de aprobación de escritura (humano-en-el-bucle) ✓

```text
El agente decide que se necesita una escritura
  → verify_write_safety → Denegada
    → orexis.request_write_approval → WriteApprovalRequest difundida
      → shittim-chest muestra diálogo de aprobación (industrial.approveWrite)
        → [aprobada] → entrada temporal en lista blanca → ejecutar + verificar lectura
        → [denegada]   → el agente recibe la denegación, ajusta el plan
```

**Implementado:**

| # | Tarea | Archivo | Estado |
| --- | --- | --- | --- |
| A.2.4.1 | Herramienta MCP `orexis.request_write_approval` — construye `WriteApprovalRequest`, difunde `TuiMessage::IndustrialWriteApprovalPush`, suspende (oneshot + timeout) hasta que el operador responda | `packages/agents/orexis/src/mcp/tools/industrial_write_tools.rs` | ✓ |
| A.2.4.2 | Manejador WS `industrial.approveWrite` — resuelve la solicitud pendiente mediante el `WriteApprovalRegistry` compartido; al aprobar añade una entrada temporal en la lista blanca para que la escritura subsiguiente pase `verify_write_safety` | `packages/scepter/src/tui_connection/mod.rs` | ✓ |

El productor/resolutor están desacoplados mediante un `WriteApprovalRegistry`
compartido a nivel de proceso (`_shared_security_policy::write_approval_registry`),
inyectado en orexis al inicio y usado por scepter cuando el operador responde.

---

## Fase E: Dogfood de Extremo a Extremo

Validación operacional, no código puro. Requiere ejecutar simuladores de hardware.

### E.1 — Entorno de prueba

| # | Componente | Configuración |
| --- | --- | --- |
| E.1.1 | Simulador S7comm | Ejecutar la crate `snap7-server` como S7-1500 virtual. Precargar DB1 con: REAL temp en offset 0, REAL pressure en offset 4, INT flow en offset 8, BOOL valve en offset 10, más 50 bytes de datos aleatorios |
| E.1.2 | Simulador Modbus | Ejecutar aoba en modo esclavo en puerto serie virtual (`socat pty pty`). Precargar estación 5 con valores de registro conocidos |
| E.1.3 | Entelecheia + evernight | Inicio estándar docker-compose. evernight `sensor-poll` listo con flag `--manifest` |

### E.2 — Escenarios de dogfood

| # | Escenario | Pasos | Criterios de aprobación |
| --- | --- | --- | --- |
| E.2.1 | **Corredor S7comm desconocido** | (1) Dar al sistema el objetivo `192.168.1.10:102`. (2) La cadena de habilidades `industrial_discover` se ejecuta autónomamente. (3) El sistema descubre el protocolo S7comm, DB1, infiere la semántica de los campos, genera el manifiesto. (4) El operador revisa el manifiesto en TUI. (5) Aprobar → evernight comienza a sondear. (6) Inyectar valor de alarma → se dispara Hubris alarm_response → se propone acción correctiva. | Manifiesto generado con ≥ 3 campos correctamente inferidos. La alarma dispara la cadena `alarm_response → task_decompose → plan_execute`. |
| E.2.2 | **Corredor Modbus desconocido** | Mismo flujo pero con Modbus RTU en puerto serie virtual. Disposición de estación diferente. | Mismos criterios. |
| E.2.3 | **Descubrimiento de protocolo mixto** | Ejecutar ambos simuladores simultáneamente. El sistema descubre ambos, genera manifiesto combinado. | Ambas estaciones aparecen en el manifiesto con protocolos correctos. |
| E.2.4 | **Flujo de aprobación de escritura** | El agente propone cerrar una válvula (escribir en el campo BOOL descubierto). `verify_write_safety` bloquea (no está en lista blanca). WriteApprovalRequest enviada al operador. El operador aprueba. La escritura se ejecuta con verificación de lectura. | Ciclo completo: proponer → bloquear → solicitar → aprobar → ejecutar → verificar. **(D.2 ahora entregado — listo para dogfood.)** |

### E.3 — Grabación de demo

| # | Tarea | Notas |
| --- | --- | --- |
| E.3.1 | Grabar el ciclo completo descubrimiento→monitorización→alarma→respuesta como captura de pantalla | Demostrar adaptación autónoma a hardware desconocido |
| E.3.2 | Generar artefacto de informe de descubrimiento (manifiesto TOML autogenerado + tabla de campos inferidos) | Entregable tangible para la revisión del hito gubernamental |

---

## Dependencia de Proyectos Hermanos (restante)

| Hermano | Qué necesitamos de ellos | Cuándo | Estado |
| --- | --- | --- | --- |
| **arona** | Ruta de difusión WS para `WriteApprovalRequest` (A.2.4) | ~~bloquea A.2.4 / D.2~~ hecho — viaja en `TuiMessage::IndustrialWriteApprovalPush` (reexportado de tipos arona) | ✓ |
| **shittim-chest** | Diálogo de aprobación del operador (`industrial.approveWrite` consumidor) + renderizado de progreso de descubrimiento | bloquea E.2.4 dogfood (el manejador WS en scepter está listo; shittim-chest necesita renderizar el diálogo y hacer POST de la respuesta) | PLAN hermano |

---

## Explícitamente Fuera del Alcance (sprint de 2 semanas)

- Cliente/servidor OPC UA (ecosistema Rust no está listo)
- EtherNet/IP / CIP (Rockwell)
- EtherCAT (Beckhoff)
- Bus CAN
- Cobertura de pruebas del frontend (shittim-chest recibe solo plan de guía, sin escritura de pruebas)
- Paridad de características del CLI con TUI

---

# Hoja de Ruta Técnica — Profundización de la Arquitectura

> **Fecha**: 2026-06-26
> **Contexto**: Después de limpiar el repositorio de más de 700 documentos/archivos obsoletos y consolidar todos los prompts en `res/prompts/`, auditamos los documentos de diseño restantes contra el código fuente real para identificar qué diseños aspiracionales vale la pena implementar.

---

## 1. Direccionamiento de Sub-Insignias + Ejecución Paralela de Habilidades

**Veredicto**: Vale la pena implementar. Infraestructura ~80% construida, falta solo el 20% final.

**Estado actual**:

- `BadgeRegistry` (`packages/scepter/src/state_machine/badge_registry.rs:92-120`) ya soporta `link_sessions()` padre-hijo.
- El parseo de sintaxis de sub-insignia `#001.005` existe en `find_by_container_id_or_sub()` pero elimina el sub-número en lugar de resolverlo a un contenedor hijo distinto.
- Los campos `SnowflakeContainer.parent_id` y `branch_level` existen pero son solo metadatos — nunca se usan para enrutamiento.
- La cola de prioridad de nodos edge (`edge_node_registry.rs:73-126`) está lista para bloqueo de recursos de grano fino.
- La cadena de habilidades es estrictamente **serial** — `pipeline.rs:68-226` itera una habilidad a la vez. Las habilidades de coordinador con `next_targets` independientes se ejecutan en serie cuando podrían ejecutarse en paralelo.

**Qué falta**:

1. ✅ Hacer que `find_by_container_id_or_sub()` resuelva `#001.005` → el hijo bifurcado activo más profundo del contenedor padre, recurriendo al padre cuando no existe bifurcación (compatible hacia atrás).
1. ✅ Añadir búsqueda de hijos/descendientes a `SnowflakeManager`: `children_of`, `children_of_badge`, `most_recent_child_of`, `deepest_descendant` (`parent_id` → índice inverso).
1. ✅ Ejecución paralela basada en `FuturesUnordered` de `next_targets`: `dispatch_parallel_targets` expande los objetivos **hoja** independientes de un coordinador concurrentemente mediante `parallel_dispatch::fan_out` (limitado por un `Semaphore`). Los dos bloqueadores singleton globales en la ruta serial `invoke_skill_with_retries` se manejan de la siguiente manera: **namespace cosmos local compartido** → cada objetivo se bifurca en su **propio contenedor cosmos** en Fase 1 (`fork_container_for_skill` + `assign_container_id` + `register_container_badge_in_registry`), por lo que `dump/restore_cosmos_namespace` es un no-op por rama y la ejecución concurrente está aislada. `MAX_BRANCH_DEPTH` (punto 4) limita la cadena de bifurcación. **Carrera de UI `active_streaming_skill`** → tolerada (último escritor gana en un `Option`; se restablece a `None` después de cada rama). **Threading de `&mut SkillChainInput`** → `BranchOwner` refleja las porciones mutables por rama; `as_input` las toma prestadas de vuelta en un `SkillChainInput` de corta duración para que los ayudantes de pipeline sin cambios se reutilicen. La Fase 1 (bifurcar + preparar + construir prompts + lista blanca de herramientas) está **serializada** para evitar carreras de `rag_buffer`; solo la Fase 2 (las invocaciones LLM dominantes en latencia) se ejecuta en paralelo; la Fase 3 limpia y fusiona informes (`merge_branch_reports`) en el contexto padre. Protegido detrás de `SKILL_CHAIN_PARALLEL_TARGETS` (por defecto **apagado**) + `parallel_targets_eligible` (contenedorizado + todos los objetivos hoja). El desenrollado de pila serial en `route_to_next_skill` sigue siendo el predeterminado.
1. ✅ Aplicar `MAX_BRANCH_DEPTH` (`COSMOS_MAX_BRANCH_DEPTH`, por defecto 4) en ambas rutas de bifurcación; los hijos ahora se registran en `source.branch_level + 1` en lugar de un `1` hardcodeado.

**Impacto esperado**: Escrituras de archivos paralelas, análisis paralelos desde habilidades de coordinador como `industrial_discover` reducirían la latencia de extremo a extremo significativamente.

---

## 2. Pipeline de Sedimentación de Memoria

**Veredicto**: Multiplicador de calidad, no crítico. Reservado para la hoja de ruta a largo plazo.

**Estado actual**:

- `PhiliaMemoryService` es un grafo plano de "almacenar → incrustar → recuperar" sin metabolismo.
- `memory_consolidate` es trivial — solo crea un nodo de episodio, sin abstracción/resumen.
- Sin decaimiento de memoria, envejecimiento, puntuación de obsolescencia o gradiente de calidad entre nodos.
- Todos los nodos son `MemoryNode` indiferenciados — sin separación episódica/procedimental/atómica.
- La búsqueda vectorial en memoria es O(n) fuerza bruta (no escalará a largo plazo).
- `KnowledgeStore` (sistema separado) tiene etapas de ciclo de vida (Created→Vectorized→Searchable→Consolidated→Deprecated) y validación de consenso — este es el análogo existente más cercano a la sedimentación.

**Por qué no es urgente**:

- La inyección de contexto RAG (`RagContextBuffer` → reescritura de consulta LLM → `bundle_search`) proporciona contexto suficiente para los agentes actuales de llamada a herramientas.
- El índice HNSW de pgvector maneja la recuperación a escala de producción.
- El sistema funciona como "almacenar y recuperar" — la sedimentación lo haría "metabolizar", pero esto es calidad incremental, no una brecha funcional.

**Trabajo futuro** (sin cronograma):

- Auto-consolidación: resumen periódico impulsado por LM de nodos relacionados en "episodios" de nivel superior.
- Gradiente de calidad: conteos de acceso, decaimiento temporal, puntuación de confianza.
- Prototipo de tres canales (episódico/procedimental/atómico) con estrategias de recuperación diferenciadas.

---

## 3. Negociación Entre Agentes

**Veredicto**: Baja prioridad. Existen primitivas como bloques de construcción de bajo nivel; sin caso de uso inmediato.

**Estado actual**:

- `deliver_message(message_type="Question")` existe (`epieikeia/src/mcp/tools/deliver_message.rs:63`) — puede enviar preguntas al buzón de otro agente.
- `inject_user_prompt` / `consume_injected_prompts` existen pero están **basados en sondeo** — sin integración en el pipeline. Los agentes deben llamar explícitamente a `consume_injected_prompts` para revisar el correo.
- `Haplotes` tiene tipos de enrutamiento de conversación `AskAgent` / `ReplyAgent` / `Escalated` — pero todos son ACKs no-op con cero lógica de negocio.
- Las variables de entorno `NEGOTIATION_ROUND_TIMEOUT_SECS` / `NEGOTIATION_TOTAL_TIMEOUT_SECS` están definidas en `RuntimeTuningConfig` pero **nunca se consumen** en ninguna parte — código muerto.

**Por qué es baja prioridad**:

- El despacho secuencial actual de cadena de habilidades + paso de contexto como cadena maneja todos los casos de uso actuales.
- Los conflictos de fusión se manejan mediante despacho de una sola habilidad (`resolve_merge_conflict`), lo cual es suficiente.
- El bucle de negociación (interceptar cadena de habilidades → preguntar al agente → esperar respuesta → incorporar) sería complejo de construir y probar. Ningún caso de uso en producción lo exige aún.

**Cuándo revisitar**: Si los agentes alguna vez necesitan negociar dinámicamente decisiones a mitad de cadena (no solo despachar y esperar), las primitivas están construidas al 40%. La brecha es el bucle de integración en el pipeline.

---

## Resumen

| Funcionalidad | Infra construida | Prioridad | Próximo paso |
| --- | --- | --- | --- |
| Sub-insignia + ejecución paralela | 100% | **Alta** | ✅ Hecho — sub-insignia→hijo, índice de hijos, profundidad de rama y despacho paralelo en bucle, todo entregado (paralelo desactivado por defecto) |
| Sedimentación de memoria | 20% | **Largo plazo** | Sin acción inmediata; revisitar después de ejecución paralela |
| Negociación entre agentes | 40% | **Baja** | Esperar caso de uso concreto; las primitivas están listas |
