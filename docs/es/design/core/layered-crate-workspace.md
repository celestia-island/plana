+++
title = "ADR-004: Arquitectura de Espacio de Trabajo en Capas de 60+ Crates"
description = """Fecha: 2026-03"""
lang = "es"
category = "design"
subcategory = "core"
+++

# ADR-004: Arquitectura de Espacio de Trabajo en Capas de 60+ Crates

**Fecha**: 2026-03
**Estado**: Aceptado

## Contexto

Entelecheia comenzó con un crate monolítico `packages/shared` (38K líneas, 187 archivos `.rs`) que contenía toda la infraestructura compartida: tipos, protocolo MCP, proveedores LLM, gestión de contenedores, base de datos, seguridad, configuración y más. A medida que el proyecto creció a 12 agentes + 1 agente de dominio + 3 paquetes binarios, surgieron varios problemas:

1. **Tiempos de compilación**: Cualquier cambio en `shared` requería recompilar todos los 187 archivos, incluso si solo se modificaba una estructura.
1. **Contaminación de dependencias**: Los crates de agente que solo necesitaban tipos MCP se veían obligados a depender transitivamente de controladores de base de datos, runtimes de contenedores y proveedores LLM.
1. **Propiedad poco clara**: Con 187 archivos en un crate, no estaba claro qué módulo "poseía" qué funcionalidad, haciendo que la refactorización fuera arriesgada.
1. **Explosión de feature flags**: La compilación condicional mediante características de Cargo se usaba para evitar incorporar dependencias innecesarias, pero esto llevó a una explosión combinatoria en configuraciones de prueba.

## Decisión

Descomponer el monolítico `packages/shared` en **37 sub-crates enfocados** organizados en **6 capas de dependencia** (L0 a L5), siguiendo una dirección de dependencia estricta:

```text
L0 (hoja) → L1 → L2 → L3 → L4 → L5 → consumidores (scepter, agentes, tui)
```

**Definiciones de capas:**

| Capa | Crates | Regla |
| --- | --- | --- |
| **L0** | core, logging, macros | Cero dependencias internas en otros crates de entelecheia |
| **L1** | domain_enums, mcp_types, text, concurrent | Dependen solo de L0 |
| **L2** | config, agent_registry, state_types | Dependen de L0-L1 |
| **L3** | domain_agent, container, agent_lifecycle, agent_runtime, thread_types, toolchain, infra_utils | Dependen de L0-L2 |
| **L4** | state_sync, domain_skills, hooks, domain_auth, container_runtime, skills_permissions, timeline, iepl | Dependen de L0-L3 |
| **L5** | llm_provider, prompt, custom_agent, storage, infra_jsonrpc, infra_services, e2e_events, adapter, plugin_host, rag, embedding, security_policy | Dependen de L0-L4 |

Todas las declaraciones de dependencia interna usan `workspace = true` para consistencia de versiones. No existe un crate agregador delgado — los consumidores importan directamente de sub-crates individuales.

## Consecuencias

### Positivas

- **Compilación incremental**: Un cambio en `shared-core` (L0) aún se propaga, pero un cambio en `shared-security-policy` (L5) solo recompila ese crate y sus consumidores directos. Los tiempos de compilación mejoraron significativamente.
- **Límites de propiedad claros**: Cada crate tiene una responsabilidad enfocada. El alcance de revisión de código está naturalmente delimitado por los límites del crate.
- **Aislamiento de dependencias**: Los crates de agente importan solo los crates compartidos que necesitan. SkeMma no incluye controladores de base de datos. EleOs no incluye runtimes de contenedores.
- **Prevención de dependencias circulares**: La arquitectura en capas hace estructuralmente imposible crear dependencias circulares — los crates L3 no pueden depender de crates L5.
- **Probable en aislamiento**: Las pruebas de cada crate se ejecutan independientemente, sin requerir el árbol de dependencias completo del espacio de trabajo.

### Negativas

- **Sobrecarga de gestión del espacio de trabajo**: 60+ crates en un solo espacio de trabajo significa más archivos `Cargo.toml` que mantener, más secciones `[dependencies]` que actualizar en cambios de versión, y declaración de dependencias más cuidadosa.
- **Refactorización entre crates es más difícil**: Mover un tipo de L2 a L3 requiere actualizar todos los consumidores de L2 y verificar que ningún crate L3+ dependa accidentalmente del tipo movido a través de la ubicación antigua.
- **Verbosidad de nombres de crate**: Los nombres de crate internos usan la convención de prefijo `_shared_*` (ej., `_shared_domain_skills_permissions`), que es verboso pero necesario para claridad en el espacio de trabajo.
- **Posible sobre-descomposición**: Algunos crates (ej., `shared-text` con ~200 líneas) pueden no justificar su propia sobrecarga de crate. La descomposición siguió una filosofía de "separar si podría crecer" en lugar de necesidad estricta.

### Compromiso Aceptado

**Complejidad de gestión por tiempo de compilación y claridad arquitectónica.** Una descomposición de 37 crates de `shared` está en el extremo agresivo del diseño de espacios de trabajo Rust. Un punto medio (10-15 crates) habría sido más simple de gestionar. Sin embargo, dada la amplia superficie del proyecto (26 proveedores LLM, 2 runtimes de contenedores, 12 agentes, pipeline de seguridad completo, base de datos, IEPL), la descomposición de grano fino asegura que cada pieza pueda evolucionar independientemente. El patrón `workspace = true` mitiga la sobrecarga de gestión de versiones.
