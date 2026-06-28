+++
title = "ADR-002: Boa como Motor JavaScript Embebido"
description = """Fecha: 2026-02"""
lang = "es"
category = "design"
subcategory = "core"
+++

# ADR-002: Boa como Motor JavaScript Embebido

**Fecha**: 2026-02
**Estado**: Aceptado

## Contexto

El IEPL (Capa de Pipeline de Ejecución Interactiva) requiere un runtime JavaScript para ejecutar código generado por LLM dentro de cada contenedor COSMOS. Este runtime debe:

1. Ser **embebible** dentro de una aplicación Rust — se ejecutará como el proceso init (PID 1) dentro de contenedores ligeros.
1. Ser **seguro y aislado** — el código generado por LLM no es confiable por naturaleza.
1. Soportar **importaciones de módulos ES** para el mecanismo de despacho de herramientas (ej., `import { file_read } from 'agent'`).
1. Tener **sobrecarga de inicio mínima** — los contenedores son efímeros y deben estar listos rápidamente.
1. Ser **multiplataforma** y fácil de compilar para el objetivo del contenedor.

Se evaluaron varios runtimes JavaScript/TypeScript:

| Runtime | Lenguaje | Embebible en Rust | Control de Sandbox | Velocidad de Inicio | Soporte de Módulos ES |
| --- | --- | --- | --- | --- | --- |
| **Boa** | Rust Puro | Nativo (crate Rust) | Completo (el host controla todo) | Rápido (<10ms) | Parcial (suficiente para IEPL) |
| **Deno** | Rust + V8 | Posible vía FFI | Limitado (aislados V8) | Lento (~100ms) | Completo |
| **QuickJS** | C | Vía FFI/bindings | Moderado | Rápido | Parcial |
| **V8** | C++ | Vía FFI (crate v8) | Limitado | Lento | Completo |
| **wasmoon** | C (Lua) | Vía FFI | Bueno | Rápido | N/A (Lua) |
| **rquickjs** | C (QuickJS) | Vía FFI | Moderado | Rápido | Parcial |

## Decisión

Elegimos **Boa Engine** (v0.21) como el runtime JavaScript embebido.

**Razones principales:**

1. **Rust puro — cero sobrecarga FFI.** Boa está escrito completamente en Rust, lo que significa que se compila nativamente en el binario COSMOS sin cadena de dependencias C/C++. Esto elimina toda una clase de vulnerabilidades de seguridad relacionadas con FFI, complejidad de compilación y dolores de cabeza de compilación cruzada. En el contexto del contenedor COSMOS donde controlamos el proceso init, esta es una ventaja crítica.

1. **Simplicidad de "invocar y usar".** Boa está diseñado como un motor de biblioteca primero. Puede ser instanciado, configurado con funciones host y ejecutado en pocas líneas de código Rust. No hay un proceso separado que gestionar, ni puente IPC que mantener, ni complejidad de bucle de eventos. El `JsReplHandle` en COSMOS crea un hilo OS dedicado para el runtime Boa y se comunica mediante canales Rust estándar — una arquitectura limpia y componible.

1. **La seguridad es la prioridad máxima, no el rendimiento.** En el sandbox COSMOS, cada milisegundo de tiempo de ejecución JS no está en el camino crítico — el cuello de botella siempre es el viaje de ida y vuelta de inferencia LLM. Lo que importa es que el runtime nos dé **control completo** sobre lo que el código ejecutado puede hacer. La API de registro de funciones host de Boa nos permite definir precisamente qué funciones están disponibles (solo las funciones de despacho de herramientas MCP), sin escapes. El validador de seguridad AST (que bloquea `eval`, `require`, `process`, etc.) opera sobre el AST de Boa, dándonos una garantía Rust nativa de aplicación.

1. **Soporte de módulos ES suficiente para IEPL.** El pipeline IEPL usa un sistema de módulos simulado — las importaciones de módulos ES se resuelven a nivel del constructor de espacio de nombres, no por un cargador de módulos real. Las capacidades de Boa son más que suficientes para este patrón. No necesitamos un algoritmo de resolución de módulos compatible con Node.js completo.

## Consecuencias

### Positivas

- **Sin dependencia de compilación C/C++** — COSMOS se compila limpiamente con solo `cargo build`, sin requisitos de bibliotecas a nivel de sistema.
- **Control de sandboxing completo** — Cada función disponible para el código JS ejecutado es registrada explícitamente por el host. Sin acceso predeterminado a E/S, red o sistema de archivos.
- **Integración estrecha con tipos Rust** — Implementación del trait `boa_gc::Trace` para objetos host personalizados, interoperabilidad nativa con `serde_json`, cero copia donde sea posible.
- **Seguridad contra fallos** — Los pánicos de Boa son capturados por el límite del hilo OS, evitando que el proceso COSMOS falle debido a código JS mal formado.
- **Huella binaria pequeña** — Comparado con soluciones basadas en V8, Boa añade significativamente menos al tamaño del binario COSMOS.

### Negativas

- **La conformidad JavaScript es incompleta** — Boa no implementa completamente ECMAScript 2024+. Algunas características avanzadas (ej., `WeakRef`, `FinalizationRegistry`, integración completa de `Promise`, `async/await` en todos los contextos) pueden tener limitaciones o implementaciones faltantes.
- **El rendimiento no es competitivo con V8/`SpiderMonkey`** — El intérprete de Boa es significativamente más lento que los motores compilados JIT. Para cargas de trabajo intensivas en CPU (procesamiento de grandes datos, algoritmos complejos), esto importa. Sin embargo, en el contexto IEPL, el código JS es principalmente pegamento de orquestación llamando herramientas MCP, no computación.
- **El ecosistema es más pequeño** — Boa tiene menos contribuyentes y menos pruebas de batalla que V8 o QuickJS. Los errores pueden tardar más en corregirse upstream.
- **Sin `eval` o generación dinámica de código** — Por diseño (y aplicado por el validador AST), la evaluación dinámica de código está bloqueada. Esto limita ciertos patrones de meta-programación pero es aceptable para el modelo de seguridad.

### Compromiso Aceptado

**Sacrificio de rendimiento por seguridad y embebibilidad.** Si el motor de ejecución IEPL necesitara ejecutar algoritmos complejos o procesar grandes conjuntos de datos, Boa sería la elección incorrecta. Pero en el sandbox COSMOS, el código JS es una capa de orquestación delgada — su trabajo es llamar herramientas MCP en el orden correcto, manejar errores y componer resultados. El trabajo pesado real lo hacen las herramientas de agente basadas en Rust. La velocidad de ejecución 10-100x más lenta de Boa comparada con V8 es irrelevante cuando cada llamada a herramienta implica un viaje de red a Scepter que toma 50-500ms.
