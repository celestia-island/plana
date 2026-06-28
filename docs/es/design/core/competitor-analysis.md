+++
title = "Análisis Competitivo de Frameworks Multi-Agente"
description = """Fecha: 12 de mayo de 2026 (actualizado tras auditoría completa del código fuente de 43 crates × 1500+ archivos fuente)"""
lang = "es"
category = "design"
subcategory = "core"
+++

# Análisis Competitivo de Frameworks Multi-Agente

**Fecha**: 12 de mayo de 2026 (actualizado tras auditoría completa del código fuente de 43 crates × 1500+ archivos fuente)
**Contexto**: Comparación estructurada contra las dimensiones de diseño de Entelecheia（玄枢）.

> Nota sobre el estado actual: las referencias a Entelecheia en este documento mezclan la realidad actual del código y la arquitectura prevista. Lea las secciones "vs. Entelecheia" como comparaciones contra los objetivos de diseño de Entelecheia, no como afirmaciones de que cada capacidad está completamente implementada hoy. Para la realidad actual de la implementación, priorice el apéndice aquí y el informe de diagnóstico del 13-05-2026.

---

## 1. CrewAI

**Repo**: [crewAIInc/crewAI](https://github.com/crewAIInc/crewAI)
**Lenguaje**: Python
**Licencia**: MIT
**Tamaño**: ~23k+ estrellas. Independiente de LangChain.

### Arquitectura

- **Agentes**: Definidos mediante YAML (rol, objetivo, historia) o clase `Agent` de Python. Cada agente envuelve un LLM con acceso a herramientas.
- **Orquestación**: Dos modos:
  - **Crews**: Equipo de agentes con procesos secuenciales o jerárquicos. El secuencial ejecuta tareas en orden; el jerárquico asigna un agente "gestor" para delegación.
  - **Flows**: DAG dirigido por eventos con decoradores `@start`, `@listen`, `@router`. Estado tipado con Pydantic. Soporta combinadores de condiciones `and_`/`or_`.
- **Comunicación**: Paso de mensajes a través del runtime Crew/Flow. Los agentes producen salida estructurada (`output_pydantic`, `output_json`).
- **Tipos de proceso**: `Process.sequential` y `Process.hierarchical`.

### Exposición de Herramientas

- Herramientas integradas mediante el paquete `crewai[tools]` (SerperDev, etc.). Herramientas personalizadas como funciones Python.
- Soporte MCP (Model Context Protocol) documentado.
- Las herramientas se asignan por agente en el momento de la definición.
- Sin límite explícito de herramientas por llamada LLM — todas las herramientas asignadas se exponen en cada turno.

### Modelo de Seguridad

- **Sin sandboxing**. Los agentes se ejecutan en el mismo proceso Python que la orquestación.
- El "AMP Suite" empresarial ofrece un plano de control con observabilidad y controles de acceso (propietario).
- Human-in-the-loop mediante `human_input=True` en las tareas.
- No se menciona aislamiento de ejecución de código.

### Memoria/Contexto

- Corto plazo: Memoria del agente mediante historial de conversación.
- Largo plazo: Almacenes de memoria opcionales habilitados mediante `memory=True` en los agentes.
- Sin compactación de contexto explícita ni gestión de tokens — depende de la ventana de contexto del LLM.
- El checkpointing se menciona en los documentos pero los detalles son escasos en OSS.

### Características Únicas

- **Sinergia Flows + Crews**: Combina equipos de agentes autónomos con flujos de trabajo precisos dirigidos por eventos.
- **Configuración YAML-first**: Agentes y tareas definidos declarativamente, adecuado para no desarrolladores.
- **Gran comunidad**: Más de 100k desarrolladores certificados a través de `learn.crewai.com`.
- **Afirmaciones de rendimiento**: 5.76x más rápido que LangGraph en ciertas tareas de QA (autoreportado).

### Brechas Potenciales Respecto a los Objetivos de Diseño de Entelecheia

- Sin sandboxing ni aislamiento de ejecución de código.
- Sin modelo de seguridad formal para la ejecución de herramientas.
- La memoria es relativamente simple — sin gestión jerárquica de contexto ni archivado.
- El runtime Python de un solo proceso limita la escalabilidad entre máquinas.
- Sin integración nativa de navegador/shell para tareas de codificación de agentes.
- La orquestación es solo Python (sin runtime multi-lenguaje).

---

## 2. LangGraph

**Repo**: [langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)
**Lenguaje**: Python (también JS/TS mediante `langgraphjs`)
**Licencia**: MIT

### Arquitectura

- **Máquina de estados basada en grafos**: Nodos (agentes/funciones) y aristas (transiciones) forman un grafo dirigido inspirado en Pregel/Beam.
- **Agentes**: Los nodos pueden ser llamadas LLM, ejecuciones de herramientas o cualquier función Python. No están fuertemente tipados como "agentes" — más bien como funciones en un grafo.
- **Orquestación**: `StateGraph` con esquema de estado tipado. Los nodos leen/escriben estado. Aristas condicionales para ramificación. Subgrafos para composición.
- **Comunicación**: El objeto de estado es la fuente única de verdad. Los mensajes se añaden a listas de estado.

### Exposición de Herramientas

- Las herramientas son herramientas de LangChain o invocables arbitrarios vinculados a nodos del grafo.
- Todas las herramientas disponibles en un nodo se exponen al LLM en ese paso.
- Sin limitación de herramientas incorporada; los desarrolladores gestionan qué herramientas se pasan por nodo.

### Modelo de Seguridad

- **Sin sandboxing**. La ejecución de código es responsabilidad del desarrollador.
- Human-in-the-loop mediante `interrupt()` — pausa la ejecución del grafo, permite inspección/modificación del estado.
- Ejecución duradera: estado persistido, puede reanudarse tras fallos (checkpointing).
- Sin primitivas de aislamiento para ejecución de herramientas o acceso al sistema de archivos.

### Memoria/Contexto

- **Corto plazo**: Memoria de trabajo mediante estado (listas de mensajes).
- **Largo plazo**: Memoria persistente entre sesiones mediante la abstracción `Store` (clave-valor con embeddings).
- La compactación de contexto no está incorporada — los desarrolladores gestionan el tamaño del estado.
- Checkpointing mediante `MemorySaver` o `SqliteSaver`.

### Características Únicas

- **Ejecución duradera**: Reanuda automáticamente desde checkpoint tras fallos/timeouts — ideal para agentes de larga duración.
- **Human-in-the-loop mediante interrupciones**: Patrón potente para flujos de aprobación.
- **Integración LangSmith**: Observabilidad profunda, trazado y evaluación.
- **Despliegue LangSmith**: Plataforma de despliegue en producción con prototipado visual.
- **Deep Agents**: Nuevo subproyecto para agentes que planifican, usan subagentes y aprovechan sistemas de archivos.
- **Ecosistema LangChain**: Integración perfecta con herramientas, modelos y componentes de LangChain.

### Brechas Potenciales Respecto a los Objetivos de Diseño de Entelecheia

- Fuertemente acoplado al ecosistema LangChain (aunque "puede usarse sin LangChain").
- Sin modelo de seguridad — sin sandboxing, sin sistema de permisos.
- Framework de bajo nivel — requiere boilerplate significativo para interacciones de agentes.
- Sin protocolo nativo de comunicación multi-agente (A2A).
- El enfoque de grafo de estados puede volverse difícil de manejar para interacciones complejas de agentes.
- Sin soporte incorporado para aislamiento de ejecución de código.

---

## 3. MetaGPT

**Repo**: [`FoundationAgents`/MetaGPT](https://github.com/`FoundationAgents`/MetaGPT)
**Lenguaje**: Python
**Licencia**: MIT
**Investigación**: Publicado en ICLR 2024

### Arquitectura

- **Multi-agente basado en SOP**: Modela una empresa de software con roles predefinidos (PM, Arquitecto, Ingeniero, etc.).
- **Agentes (Roles)**: Cada `Role` tiene un perfil, objetivo, restricciones y un conjunto de `Action`s. Los roles usan bucles ReAct (pensar → actuar) con tres modos: `REACT`, `BY_ORDER`, `PLAN_AND_ACT`.
- **Orquestación**: La clase `Team` contrata roles, invierte presupuesto, ejecuta rondas. `Environment` gestiona el paso de mensajes entre roles mediante publicación-suscripción.
- **Comunicación**: Paso de mensajes basado en pub/sub a través de Environment. Los roles se suscriben a etiquetas de mensajes específicas.
- **Filosofía central**: `Code = SOP(Team)` — Procedimientos Operativos Estándar materializados.

### Exposición de Herramientas

- Las acciones son clases Python predefinidas (`WriteCode`, `DesignAPI`, `DebugError`, etc.) — ~40+ tipos de acción.
- Cada rol recibe acciones específicas asignadas en la construcción.
- Las herramientas incluyen: motores de búsqueda web (Serper, SerpAPI, DuckDuckGo, Google, Bing), navegadores web (Playwright, Selenium), generación de imágenes (DALL-E), almacenes de documentos (Chroma, FAISS, Milvus, LanceDB, Qdrant).
- El LLM ve solo los esquemas de acción para sus acciones actualmente asignadas, no el conjunto completo de herramientas.

### Modelo de Seguridad

- **Sin sandboxing**. El código se genera y ejecuta en el mismo entorno.
- Dockerfile proporcionado pero para despliegue, no para aislamiento por tarea.
- Seguimiento de presupuesto: el parámetro `investment` limita el coste total de API LLM, lanza excepción cuando se excede.
- Sin human-in-the-loop durante la ejecución.
- Sin sandboxing de ejecución de herramientas ni modelo de permisos.

### Memoria/Contexto

- **Corto plazo**: Clase `Memory` en `RoleContext` — lista ordenada de mensajes por rol.
- **Largo plazo**: Clases `LongTermMemory` y `BrainMemory` para conocimiento persistente.
- **Memoria de trabajo**: Memoria de trabajo separada para operaciones del planificador.
- **Búfer de mensajes**: Cola de mensajes asíncrona con filtrado por etiquetas suscritas.
- La compresión de contexto no se maneja explícitamente — los roles observan un subconjunto filtrado de mensajes.

### Características Únicas

- **Emulación completa del SDLC**: Modela toda la empresa de software con SOPs — historias de usuario, requisitos, documentos de diseño, código, pruebas.
- **Múltiples almacenes de documentos**: Más de 5 opciones de BD vectorial.
- **Soporte extenso de proveedores**: Más de 12 proveedores LLM (OpenAI, Azure, Anthropic, Gemini, Ollama, Bedrock, etc.).
- **Data Interpreter**: Agente especializado para tareas de ciencia de datos.
- **Resultados de investigación**: Múltiples artículos publicados (AFlow, SPO, SELA, FACT, Data Interpreter).
- **MGX**: Producto de programación en lenguaje natural construido encima.

### Brechas Potenciales Respecto a los Objetivos de Diseño de Entelecheia

- SOP rígido — roles y acciones están predefinidos; los roles personalizados requieren codificación.
- Sin sandboxing de seguridad en absoluto.
- Arquitectura de una sola máquina — sin despliegue distribuido de agentes.
- Sin control de agente basado en navegador (solo CLI/API).
- El modelo de memoria es bastante básico, colas por rol.
- Sin soporte de protocolo inter-agente MCP/A2A.
- El seguimiento de presupuesto es solo de coste, no basado en recursos/seguridad.

---

## 4. ChatDev 2.0 (DevAll)

**Repo**: [OpenBMB/ChatDev](https://github.com/OpenBMB/ChatDev)
**Lenguaje**: Python (backend), Vue 3 (frontend)
**Licencia**: Apache 2.0
**Investigación**: Múltiples artículos en NeurIPS/arxiv

### Arquitectura

- **Plataforma multi-agente sin código**: Agentes y flujos de trabajo definidos enteramente en configuración YAML. No se requiere codificación.
- **DAG de flujo de trabajo dirigido por YAML**: Los nodos definen agentes, las aristas definen el flujo de mensajes. Soporta subgrafos. Lienzo visual drag-and-drop en la UI web.
- **Módulos principales**: `runtime/` (ejecución de agentes), `workflow/` (orquestación DAG), `entity/` (configuración), `server/` (FastAPI + WebSocket), `frontend/` (consola web Vue 3).
- **Agentes**: Definidos en configuraciones de nodo YAML con prompts, configuración LLM, herramientas y ajustes de memoria.
- **Orquestación**: Múltiples tipos de ejecutor: secuencial, DAG, paralelo, ciclo, arista dinámica. El constructor de topología convierte configuraciones YAML en grafos ejecutables.

### Exposición de Herramientas

- **Sistema de llamada a funciones**: `functions/function_calling/` contiene herramientas integradas (`code_executor`, file, weather, web, video, `deep_research`, uv, user).
- **Registro de herramientas personalizadas**: Las funciones Python en el directorio `functions/` se autodescubren.
- **Soporte MCP**: `mcp_example/mcp_server.py` demuestra la integración MCP.
- Las herramientas se asignan por nodo en la configuración YAML.

### Modelo de Seguridad

- **Ejecución de código**: `code_executor.py` dedicado con parámetros de ejecución configurables.
- Despliegue Docker Compose disponible.
- **Human-in-the-loop**: Flujo de trabajo `demo_human.yaml`, nodos de entrada de usuario, flujos de confirmación.
- **Streaming WebSocket**: Monitoreo de registros en tiempo real e inspección de artefactos.
- Sin modelo explícito de sandboxing/aislamiento para agentes.

### Memoria/Contexto

- **Múltiples backends de memoria**: Memoria simple, memoria `mem0` (persistente, aprendible), memoria basada en archivos.
- **Configuración de memoria en YAML**: `store`, `context_window_size`, tipo de memoria por nodo.
- **Nodos de reinicio de contexto**: Nodos de flujo de trabajo `context_reset` explícitos.
- **Consistencia de embeddings de memoria**: Pruebas de consistencia de embeddings entre sesiones.
- **Escaneo de espacio de trabajo**: `workspace_scanner.py` para inyección de contexto basada en archivos.

### Características Únicas

- **Sin código**: Construye sistemas multi-agente sin escribir código Python — YAML + UI web.
- **Consola web drag-and-drop**: Diseñador visual de flujos de trabajo, monitoreo de lanzamiento en tiempo real.
- **Plantillas de flujo de trabajo ricas**: Visualización de datos, generación 3D (Blender), desarrollo de juegos, investigación profunda, video educativo.
- **Soporte de subgrafos**: Subflujos de trabajo reutilizables (`react_agent.yaml`, `reflexion_loop.yaml`).
- **Integración OpenClaw**: Puede ser invocado por agentes de codificación OpenClaw para crear dinámicamente equipos de agentes.
- **SDK Python**: Paquete PyPI `chatdev` para ejecución programática de flujos de trabajo.
- **Linaje de investigación**: MacNet, Puppeteer, IER, co-aprendizaje experiencial — todos construidos sobre ChatDev.
- **E-book multi-agente**: Colección curada de investigación multi-agente.

### Brechas Potenciales Respecto a los Objetivos de Diseño de Entelecheia

- Centrado en YAML limita la expresividad para lógica compleja.
- Sin sandboxing de seguridad para ejecución de código.
- La consola web es la interfaz principal — menos adecuada para uso headless/embebido.
- Similar a Django: estructura de proyecto opinada, menos flexible que los frameworks nativos de Python.
- Sin soporte de agentes multi-lenguaje.
- La comunidad está orientada a la investigación más que a la empresa.

---

## 5. Google ADK (Agent Development Kit)

**Repo**: [google/adk-python](https://github.com/google/adk-python)
**Lenguaje**: Python (también ediciones Java, Go)
**Licencia**: Apache 2.0

### Arquitectura

- **Code-first**: Agentes, herramientas y orquestación definidos en código Python.
- **Abstracciones fundamentales**: Agent (plano), Tool (capacidad), Runner (motor), Session (estado de conversación), Memory (recuerdo entre sesiones), Artifact Service (archivos).
- **Tipos de agente**: `LlmAgent` (dirigido por LLM), `LoopAgent`, `SequentialAgent`, `ParallelAgent`, `RemoteA2aAgent`.
- **Composición multi-agente**: El agente padre tiene lista `sub_agents`. El Runner maneja el enrutamiento entre agentes.
- **Protocolo A2A**: Soporte nativo para el protocolo de comunicación Agente-a-Agente para agentes remotos.
- **Integración LangGraph**: `langgraph_agent.py` para embeber grafos LangGraph como agentes.

### Exposición de Herramientas

- **Ecosistema rico de herramientas**: Más de 50 herramientas integradas — Google Search, BigQuery, Bigtable, Spanner, PubSub, Vertex AI Search, herramientas MCP, herramientas OpenAPI, herramientas LangChain, herramientas CrewAI, uso de computadora, bash, ejecución de código, Google APIs.
- **Tipos de herramientas**: `FunctionTool`, `AgentTool` (envuelve agente como herramienta), `MCPTool`, `OpenAPITool`, `LangChainTool`, `CrewAiTool`, `SkillToolset`.
- **Confirmación de herramienta (HITL)**: Flujo de confirmación explícito antes de la ejecución de la herramienta con entrada personalizada.
- **Patrón Toolbox**: `toolbox_toolset.py` para agrupar herramientas.

### Modelo de Seguridad

- **Sandboxing de ejecución de código**: Múltiples ejecutores — `container_code_executor.py`, `unsafe_local_code_executor.py`, `vertex_ai_code_executor.py`, `agent_engine_sandbox_code_executor.py`, `gke_code_executor.py`.
- **Sistema de autenticación**: Flujo OAuth2 completo, gestión de credenciales, preprocesadores de autenticación, `authenticated_function_tool.py`.
- **Human-in-the-loop**: Confirmación de herramienta, soporte de interrupción.
- **Skills**: Capacidades de agente empaquetables y versionadas con contexto de ejecución separado.
- **Identidad de agente**: Integración `agent_identity/` para cuentas de servicio.

### Memoria/Contexto

- **Session**: Historial completo de conversación por sesión, persistido mediante `SessionService` (en memoria, SQLite, PostgreSQL, Vertex AI).
- **Memory**: Recuerdo entre sesiones mediante `MemoryService` — en memoria, Vertex AI Memory Bank, Vertex AI RAG.
- **Compactación de contexto**: `compaction.py` y `llm_event_summarizer.py` para resumen automático de contexto.
- **Artifact service**: Manejo separado de datos no textuales (archivos, imágenes).
- **Caché de contexto**: `gemini_context_cache_manager.py` para caché de contexto específico de Gemini.
- **Rewind**: Capacidad de rebobinar la sesión a antes de una invocación previa.

### Características Únicas

- **Soporte multi-lenguaje**: Ediciones Python, Java, Go de ADK.
- **Despliegue en producción**: `adk deploy` a Cloud Run, Vertex AI Agent Engine, GKE.
- **Protocolo A2A**: Nacido de Google — comunicación multi-proveedor nativa entre agentes.
- **ADK Web**: UI de desarrollo/depuración integrada con trazado de eventos.
- **Framework de evaluación**: Pruebas multicapa (unitarias, integración, evals) con puntuación de trayectoria.
- **Sistema de plugins**: Registro de depuración, filtrado de contexto, resultados multimodales, guardar artefactos, analíticas BigQuery.
- **Sistema de planificación**: Planificador integrado con patrón planificar-luego-ejecutar.
- **Soporte Vibe coding**: `llms.txt` y `llms-full.txt` para contexto LLM sobre ADK.
- **Sistema de skills**: Capacidades de agente reutilizables y empaquetables.
- **Optimización de agentes**: `agent_optimizer.py` y optimizador de prompts GEPA.

### Brechas Potenciales Respecto a los Objetivos de Diseño de Entelecheia

- Fuerte dependencia de Google Cloud para funciones de producción.
- Optimizado para Gemini (aunque agnóstico al modelo).
- Base de código compleja con muchas capas de abstracción.
- Solo FastAPI para servir API (sin frameworks alternativos).
- Sin experiencia de agente nativa CLI (enfocado en UI web).
- El manejo de memoria/contexto es consciente de la caché de Gemini, menos genérico para otros modelos.

---

## 6. OpenAI Swarm (experimental, ahora deprecado)

**Repo**: [openai/swarm](https://github.com/openai/swarm)
**Lenguaje**: Python
**Licencia**: MIT
**Estado**: Reemplazado por [OpenAI Agents SDK](https://github.com/openai/openai-agents-python)

### Arquitectura

- **Primitivas minimalistas**: `Agent` (instrucciones + funciones) y handoffs (devolver otro Agent desde una función).
- **Bucle principal** (~300 líneas en `swarm/core.py`):

  1. Obtener completación del Agent actual
  1. Ejecutar llamadas a herramientas, añadir resultados
  1. Cambiar de Agent si una función devuelve un Agent
  1. Actualizar variables de contexto
  1. Repetir hasta que no haya más llamadas a herramientas o se alcance `max_turns`

- **Agentes**: Solo un nombre, modelo, instrucciones (string o invocable), lista de funciones. Sin jerarquía de agentes — delegación plana mediante handoffs.
- **Comunicación**: Mensajes de la API Chat Completions. Sin estado entre llamadas `client.run()`.

### Exposición de Herramientas

- Las herramientas son funciones Python simples. Auto-esquema desde type hints y docstrings.
- Todas las funciones asignadas a un Agent se exponen a cada llamada LLM.
- Si una función devuelve un `Agent`, la ejecución se transfiere (handoff).
- El parámetro `context_variables` se auto-rellena si está definido en la firma de la función.

### Modelo de Seguridad

- **Ninguno**. Sin sandboxing, sin aislamiento, sin modelo de permisos. Las herramientas se ejecutan en el proceso del llamante.
- Proyecto educativo/experimental explícitamente no para producción.

### Memoria/Contexto

- **Sin estado**: Sin estado entre llamadas `client.run()`. El usuario debe pasar `messages` y recibirlos de vuelta.
- **Variables de contexto**: Diccionario simple pasado a través de llamadas a funciones — puede ser leído/escrito por funciones de herramienta.
- Sin memoria, sin persistencia de sesión, sin compactación de contexto.

### Características Únicas

- **Simplicidad extrema**: Todo el framework son ~4 archivos fuente. Excelente para aprender orquestación de agentes.
- **Patrón handoff**: Elegante — un Agent es solo "instrucciones + herramientas" y los agentes delegan devolviendo otro Agent desde una función de herramienta.
- **Soporte de streaming**: Streaming incorporado con marcadores `{"delim":"start"}` / `{"delim":"end"}` para límites de agente.

### Brechas Potenciales Respecto a los Objetivos de Diseño de Entelecheia

- Deprecado (sucedido por OpenAI Agents SDK).
- Sin modelo de seguridad en absoluto.
- Sin estado/persistencia — completamente sin estado.
- Solo OpenAI (API Chat Completions).
- Sin comunicación multi-agente más allá de handoffs.
- Experimental/educativo — no es un framework de producción.

---

## 7. Cline

**Repo**: [cline/cline](https://github.com/cline/cline)
**Lenguaje**: TypeScript (extensión VS Code) + Go (CLI)
**Licencia**: Apache 2.0

### Arquitectura

- **Extensión VS Code** + CLI independiente. La extensión es la interfaz principal; la CLI es más reciente.
- **Bucle principal** (en `src/core/`):

  1. Analizar tarea del usuario (texto + imágenes)
  1. Analizar espacio de trabajo (ASTs, búsqueda regex, lecturas de archivos)
  1. Ejecutar herramientas en un bucle: crear/editar archivos, comandos de terminal, acciones de navegador
  1. Monitorear salidas (errores de linter, salida de terminal, capturas de pantalla del navegador)
  1. Auto-corregir problemas, iterar hasta completar la tarea

- **Conjunto de herramientas**: Operaciones de archivo (crear, editar, diff), comandos de terminal (con integración de shell), navegador (headless, click/type/scroll), herramientas MCP.
- **System prompt**: Ingeniería de prompts detallada con instrucciones de gestión de contexto.

### Exposición de Herramientas

- Conjunto fijo de herramientas integradas: `read_file`, `write_to_file`, `replace_in_file`, `execute_command`, `browser_action`, `use_mcp_tool`, etc.
- **Extensión MCP**: Puede crear/instalar nuevos servidores MCP bajo demanda ("añade una herramienta que..."). También se soportan servidores MCP de la comunidad.
- **@-menciones**: `@url`, `@problems`, `@file`, `@folder` para inyección de contexto (reducir costes de API).
- Número limitado de herramientas por llamada — típicamente 8-12 integradas + herramientas MCP.

### Modelo de Seguridad

- **Human-in-the-loop para todas las acciones**: Cada cambio de archivo y comando de terminal debe ser aprobado por el usuario en la GUI.
- **Sistema de checkpoint**: Instantáneas del espacio de trabajo antes de cada paso. Se puede hacer diff/restaurar en cualquier punto.
- **Sistema de permisos**: `CommandPermissionController` con listas de permitir/denegar.
- **Cline Ignore**: Archivo `.clineignore` para excluir archivos sensibles.
- **Sin sandboxing**: Los comandos de terminal se ejecutan en el entorno real del usuario — poder y riesgo.
- **Empresarial**: SSO, pistas de auditoría, VPC/enlace privado, autoalojado/on-prem.

### Memoria/Contexto

- **Gestión de contexto**: Análisis AST del proyecto, búsqueda regex de archivos relevantes, selección cuidadosa de lo que entra en la ventana de contexto.
- **Compactación de contexto**: Cuando el contexto se llena, se generan resúmenes y la conversación antigua se comprime.
- **Checkpoints**: Instantáneas completas del espacio de trabajo para reversión.
- **Estado de tarea**: `TaskState.ts` gestiona la memoria de conversación y el estado del espacio de trabajo.
- **Detección de bucles**: `loop-detection.ts` previene bucles infinitos de llamadas a herramientas.

### Características Únicas

- **Integración completa con IDE**: Vive dentro de VS Code, ve todo tu espacio de trabajo.
- **Automatización de navegador**: Capacidad Claude Computer Use para pruebas/depuración web.
- **Bucle de auto-corrección**: Monitorea errores de linter/compilador y los auto-corrige sin intervención del usuario.
- **Creación de herramientas MCP**: Puede crear nuevos servidores MCP sobre la marcha basándose en solicitudes del usuario.
- **Proceed While Running**: Ejecución de comandos de terminal no bloqueante.
- **Multi-modelo**: OpenRouter, Anthropic, OpenAI, Gemini, Bedrock, Azure, Vertex, Cerebras, Groq, Ollama, LM Studio.
- **Go CLI**: Opción CLI independiente para entornos sin VS Code.
- **Framework de evaluación**: Pruebas de 3 capas (contract, smoke, E2E/bench).

### Brechas Potenciales Respecto a los Objetivos de Diseño de Entelecheia

- Vinculado a VS Code para la experiencia completa (CLI es más reciente y menos madura).
- Arquitectura de agente único — sin colaboración multi-agente.
- Sin roles/especialización de agentes definidos.
- Se requiere aprobación humana para cada acción (aunque automatizable).
- Sin operación autónoma de larga duración sin interacción.
- No es un framework — es una aplicación. No se puede embeber como biblioteca.
- Solo TypeScript/Go, sin SDK Python.

---

## 8. Aider

**Repo**: [Aider-AI/aider](https://github.com/Aider-AI/aider)
**Lenguaje**: Python
**Licencia**: Apache 2.0

### Arquitectura

- **Programación en pareja basada en terminal**: Herramienta CLI que edita código en tu repositorio.
- **Bucle principal** (`base_coder.py`):

  1. Construir mapa del repo (resumen del código base basado en AST Tree-sitter)
  1. Enviar prompt + mapa del repo + archivos al LLM
  1. Analizar la respuesta del LLM en busca de instrucciones de edición (diffs unificados, bloques search/replace, reescrituras de archivos completos)
  1. Aplicar ediciones, lint, ejecutar pruebas, auto-corregir fallos
  1. Git commit con mensajes sensatos

- **Múltiples formatos de edición**: `udiff`, `editblock`, `wholefile`, `search_replace`, `diff_fenced`, `editor_editblock`, `editor_whole`, `patch`, `architect`. Cada uno es una clase coder separada con su propia plantilla de prompt.
- **Modo Arquitecto/Editor**: Patrón de dos agentes — el arquitecto planifica, el editor implementa. Demostrado altamente efectivo en benchmarks.

### Exposición de Herramientas

- **Sin llamada a herramientas tradicional**. Aider usa respuestas de texto estructuradas (no function calling).
- El LLM produce instrucciones de edición en formatos específicos (bloques diff, search/replace) que Aider analiza y aplica.
- Comandos de shell: El LLM puede solicitar ejecutar comandos de shell (confirmados por el usuario).
- Web scraping: Navegador basado en Playwright para leer páginas web.
- Entrada de voz: Speech-to-code mediante micrófono.
- Imágenes: Puede añadir capturas de pantalla/imágenes al chat para contexto visual.

### Modelo de Seguridad

- **Sin sandboxing**. Las ediciones se aplican directamente a tus archivos. Git proporciona red de seguridad.
- **Confirmación del usuario**: Los comandos de shell requieren aprobación explícita del usuario.
- **Integración Git**: Todos los cambios se auto-confirman — fácil reversión.
- **Modo watch**: Puede observar cambios de archivos y auto-reaplicar si las pruebas fallan.
- **Linting/testing**: Auto-ejecuta linters y suites de pruebas configurados después de los cambios.
- Sin sistema de permisos, sin aislamiento, sin acceso basado en roles.

### Memoria/Contexto

- **Mapa del repo**: El análisis AST Tree-sitter construye un mapa conciso de todo el código base (firmas de funciones, definiciones de clases, relaciones de importación). Esto ajusta la estructura del código base en el contexto sin incluir todo el código fuente.
- **Historial de chat**: Conversación completa en contexto.
- **Selección de archivos**: El LLM solicita archivos específicos mediante sintaxis — solo los contenidos de esos archivos se añaden al contexto.
- **Gestión de ventana de contexto**: Al acercarse al límite de contexto, Aider descarta turnos de conversación más antiguos e incluye resúmenes.
- **Sin memoria a largo plazo**: Sin estado entre sesiones. Cada lanzamiento de `aider` comienza fresco.

### Características Únicas

- **Mapa del repo**: Comprensión del código base basada en AST que supera el RAG basado en embeddings para tareas de código.
- **Múltiples formatos de edición**: Se adapta a lo que funciona mejor para cada modelo (algunos modelos funcionan mejor con udiff, otros con search/replace, etc.).
- **Modo Arquitecto/Editor**: Proceso de dos pasos con llamadas LLM separadas para planificación y ejecución.
- **Políglota**: Más de 100 lenguajes de programación mediante Tree-sitter.
- **Tablas de clasificación LLM**: Mantiene benchmarks públicos para edición de código entre modelos.
- **Codificación por voz**: Speech-to-code directamente en terminal.
- **Modo copiar-pegar**: Funciona con interfaces de chat web para modelos sin acceso API.
- **Consciente de Git**: Auto-commits, mensajes de commit sensatos, funciona con repositorios git existentes.
- **Auto-escritura**: El 88% del propio código de Aider fue escrito por Aider.

### Brechas Potenciales Respecto a los Objetivos de Diseño de Entelecheia

- **Enfoque en un solo archivo**: Principalmente edita un archivo a la vez (aunque el mapa del repo proporciona contexto).
- **Sin sistema multi-agente**: Solo par arquitecto/editor. Sin roles de agente personalizables.
- **Sin ecosistema de herramientas**: No puede usar APIs, bases de datos, servicios web — solo edición de archivos y shell.
- **Sin sandboxing**: Acceso directo al sistema de archivos — potente pero arriesgado.
- **Sin estado persistente**: Cada sesión independiente. Sin aprendizaje entre sesiones.
- **Sin primitivas de orquestación**: No es un framework — es una herramienta independiente.
- **Fragilidad del parseo de texto**: Depende de que el LLM produzca formatos de edición precisos — puede fallar con desviaciones del modelo.
- **Solo terminal**: No se puede embeber como biblioteca en la mayoría de flujos de trabajo (aunque existe API Python).

---

## Tabla Comparativa Resumen

| Dimensión | CrewAI | LangGraph | MetaGPT | ChatDev 2.0 | Google ADK | OpenAI Swarm | Cline | Aider |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Lenguaje** | Python | Python/TS | Python | Python/Vue | Python/Java/Go | Python | TS/Go | Python |
| **Framework/Biblioteca** | Framework | Framework | Framework | Plataforma | Framework | Experimento | Aplicación | Aplicación |
| **Arquitectura** | Crews+Flows | StateGraph | Roles basados en SOP | DAG YAML | Runner+Session | Bucle Handoff | Bucle de herramientas | Analizador de ediciones |
| **Multi-agente** | Sí (secuencial/jerárquico) | Sí (subgrafos) | Sí (basado en roles) | Sí (nodos YAML) | Sí (sub_agents) | Sí (handoffs) | No (único) | No (único+arquitecto) |
| **Sandboxing de código** | Ninguno | Ninguno | Ninguno | Mínimo | Sí (GKE, Container, Vertex AI) | Ninguno | Ninguno (reversión por checkpoint) | Ninguno (reversión git) |
| **HITL** | Sí (flag de tarea) | Sí (interrupt) | No | Sí (nodos de flujo) | Sí (confirmación de herramienta) | No | Sí (cada acción) | Sí (confirmación shell) |
| **Modelo de memoria** | Corta + larga opcional | Corta + larga (Store) | Corta + larga + trabajo | Corta + mem0 + archivo | Sesión + entre sesiones | Ninguna (sin estado) | Estado de tarea + checkpoints | Solo historial de chat |
| **Gestión de contexto** | Ninguna explícita | Ninguna explícita | Filtrado de mensajes | Ventana configurable | Auto-compactación | Ninguna | Análisis AST + compactación | Mapa del repo + auto-eliminación |
| **Exposición de herramientas** | Por agente | Por nodo | Por rol (acciones) | Por nodo YAML | 50+ integradas | Por agente (plano) | 8-12 fijas | Shell + formatos de edición |
| **Soporte de protocolos** | Ninguno | Ninguno | Ninguno | MCP | A2A, MCP, OpenAPI | Ninguno | MCP | Ninguno |
| **Soporte de modelos** | Multi-modelo | Ecosistema LangChain | 12+ proveedores | Configurable | Optimizado Gemini, multi | Solo OpenAI | 10+ proveedores | 10+ proveedores |
| **Listo para producción** | Sí (con AMP) | Sí (LangSmith Deploy) | Limitado | Limitado | Sí | No | Sí (Empresarial) | Sí |
| **Fortaleza única** | Dualidad Flows+Crews | Ejecución duradera | Emulación SDLC completa | Constructor visual sin código | Protocolo A2A, ejecución sandboxed | Elegancia minimalista | Integración IDE, automatización navegador | Mapa del repo, formatos de edición |
| **Tamaño de código** | ~50+ archivos fuente | Grande | ~200+ archivos fuente | ~150+ archivos fuente | ~200+ archivos fuente | ~4 archivos fuente | ~300+ archivos fuente | ~100 archivos fuente |

---

## Conclusiones Clave para la Hoja de Ruta de Entelecheia

1. **Seguridad/Sandboxing**: Casi todos los frameworks carecen de ejecución sandboxed. Google ADK es la excepción notable con sandboxing basado en contenedores/Kubernetes. Esta es una gran oportunidad diferenciadora.

1. **Comunicación multi-agente**: Solo Google ADK tiene un protocolo formal inter-agente (A2A). La mayoría de los frameworks usan paso de mensajes ad-hoc. Un protocolo estandarizado (como A2A) es una brecha.

1. **Arquitecturas de memoria**: La mayoría de los frameworks tienen memoria básica a corto plazo. Pocos tienen memoria jerárquica sofisticada (trabajo, corto plazo, largo plazo) con gestión automática de contexto. El filtrado de MetaGPT y la compactación de ADK son los mejores ejemplos.

1. **Gestión de exposición de herramientas**: Todos los frameworks exponen todas las herramientas al LLM por llamada. Ningún framework subconjunta dinámicamente herramientas según contexto/estado/nivel de seguridad. Esta es una brecha arquitectónica.

1. **Ejecución de código**: Solo ADK tiene ejecución de código sandboxed de grado de producción. ChatDev tiene un ejecutor de código básico. Cline/Aider dependen del entorno nativo. Esta es una brecha de seguridad crítica en todo el ecosistema.

1. **Evaluación**: ADK y Cline tienen frameworks de evaluación formales. Otros dependen de pruebas ad-hoc o benchmarks de investigación. La evaluación embebida es un diferenciador.

1. **La deprecación de OpenAI Swarm** hacia el OpenAI Agents SDK señala una tendencia del mercado hacia frameworks de grado de producción sobre experimentos educativos.

1. **La ejecución duradera de LangGraph** es excepcionalmente potente para agentes de larga duración — la mayoría de los frameworks asumen tareas de corta duración.

1. **El enfoque sin código de ChatDev 2.0** apunta a un perfil de usuario fundamentalmente diferente (no desarrolladores) que la mayoría de los frameworks. Esto es ortogonal al diseño developer-first de Entelecheia.

1. **Cline y Aider** son aplicaciones, no frameworks. Demuestran el poder de la integración estrecha de herramientas (IDE, git, terminal, navegador) pero no pueden componerse en sistemas de agentes más grandes.

---

## Apéndice: Recordatorio del Estado Actual de Entelecheia

Este apéndice es la capa de corrección autoritativa para las comparaciones anteriores.

### Qué está activo hoy

- 12 agentes de Capa 1 están compilados en el espacio de trabajo.
- 1 crate de Capa 2 para Automatización Web está activo.
- Agentes especializados adicionales están archivados como planes o documentos parciales y no deben leerse como módulos de runtime completamente implementados.

### Qué está relativamente maduro

- `packages/scepter`, `packages/shared` y `packages/tui`
- Modelo de exposición de herramientas solo ejecución
- Rutas de ejecución respaldadas por contenedores
- Almacenamiento cifrado de claves de proveedor y fontanería de autenticación relacionada con RBAC

### Qué está todavía parcial

- WebUI comparada con TUI
- Cobertura de comandos CLI
- Integraciones de escritorio/móvil (migradas a [shittim-chest](https://github.com/celestia-island/shittim-chest))
- RAG y memoria, que actualmente dependen de documentos en memoria, embeddings basados en hash y recorrido de grafos en lugar de una pila ONNX + pgvector completamente integrada
- Completitud de auditoría y endurecimiento de contenedores
