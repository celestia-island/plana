+++
title = "ADR-001: Superficie de Herramientas de Microkernel Solo-Ejecución"
description = """Fecha: 2026-02"""
lang = "es"
category = "design"
subcategory = "core"
+++

# ADR-001: Superficie de Herramientas de Microkernel Solo-Ejecución

**Fecha**: 2026-02
**Estado**: Aceptado (modificado 2026-06 — superficie reducida de 5 a 3 primitivas)

## Contexto

En un sistema de orquestación multi-agente con LLM, el modelo debe decidir qué herramientas llamar y cómo componerlas. El enfoque ingenuo es exponer cada herramienta MCP (118+ en 12 agentes) directamente al LLM como definiciones de función separadas en el prompt.

Esto crea varios problemas:

1. **Consumo de ventana de contexto**: 118+ definiciones de herramientas consumen miles de tokens, dejando menos espacio para razonamiento y conversación.
1. **Superficie de seguridad**: Cada herramienta expuesta al LLM es un vector de ataque potencial para inyección de prompts o jailbreaking.
1. **Fragmentación de aplicación de permisos**: Si las herramientas se despachan directamente por la salida del LLM, cada herramienta debe validar permisos independientemente — llevando a aplicación inconsistente y brechas.
1. **Confusión del modelo**: La investigación muestra que el rendimiento del LLM se degrada cuando se presentan demasiadas opciones de herramientas (el problema de "sobrecarga de herramientas").

## Decisión

Adoptamos un diseño de **microkernel solo-ejecución**. El LLM ve exactamente **3 primitivas de ejecución** como su superficie de herramientas:

| Primitiva | Propósito |
| --- | --- |
| `exec` | Ejecutar código TypeScript/JavaScript a través del pipeline IEPL |
| `write_to_var` | Escribir un valor de cadena en una variable REPL nombrada |
| `write_to_var_json` | Escribir un valor JSON en una variable REPL nombrada |

> **Modificación (2026-06)**: El diseño original exponía 5 primitivas, incluyendo `ref_add` y `ref_remove` para gestionar variables de referencia nombradas. Estas dos fueron **eliminadas** de la superficie visible al LLM porque el mecanismo de variable de referencia no era utilizado por ningún SOP de habilidad y añadía sobrecarga de contexto sin valor. La función `agent_allowed_tools()` en `packages/shared/domain_skills/src/tool_names.rs` ahora devuelve solo las tres primitivas anteriores. Consulte el historial de commits (`60d58f794`, `31cf9a00e`).

Todas las 118+ herramientas MCP de agentes se invocan **indirectamente** a través de importaciones de módulos ES dentro del código `exec`. El LLM genera código TypeScript que importa y llama herramientas de agente; este código es transpilado por SWC, validado por el verificador de seguridad AST, y ejecutado por el motor Boa JS dentro del sandbox COSMOS.

## Consecuencias

### Positivas

- **Sobrecarga de contexto mínima**: 3 definiciones de herramientas vs 118+. El LLM puede enfocarse en razonamiento en lugar de mecánicas de selección de herramientas.
- **Seguridad centralizada**: Todas las llamadas a herramientas pasan por el `McpRouter` que aplica listas de permitidos, doble autorización, niveles de confianza y evaluación dinámica de riesgo en un único punto de control.
- **Flujos de trabajo componibles**: El LLM puede escribir composiciones de herramientas arbitrariamente complejas en TypeScript (bucles, condicionales, manejo de errores) en lugar de estar limitado a llamadas de herramienta únicas.
- **Ejecución auditable**: Cada llamada `exec` pasa por validación AST que rechaza construcciones peligrosas (`eval`, `require`, `process`, `Function`, `import()`, acceso a `globalThis`).
- **Migración de modelo más fácil**: Los nuevos proveedores LLM solo necesitan soportar llamadas a función con 3 herramientas, no 118.

### Negativas

- **Sobrecarga de indirección**: Las llamadas a herramientas pasan por generación TypeScript → transpilación SWC → validación AST → ejecución Boa → despacho MCP router, añadiendo latencia a cada llamada.
- **Dependencia de calidad de código del LLM**: La efectividad del sistema depende de la capacidad del LLM para generar código TypeScript correcto que importe y llame correctamente las herramientas de agente.
- **Complejidad de depuración**: Cuando una llamada a herramienta falla, la cadena de error abarca generación TypeScript, transpilación, validación, ejecución JS y despacho MCP — haciendo la depuración más difícil que la invocación directa de herramientas.
- **No todos los LLM son iguales**: Los modelos de menor capacidad pueden tener dificultades con la generación de código para flujos de trabajo multi-herramienta complejos en comparación con llamadas a función simples.

### Riesgos Mitigados

- Ataques de inyección de prompts que intentan invocar directamente herramientas peligrosas
- Uso indebido accidental de herramientas debido a confusión del LLM entre demasiadas opciones
- Aplicación inconsistente de permisos entre herramientas
