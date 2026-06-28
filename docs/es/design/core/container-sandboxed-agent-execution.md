+++
title = "ADR-005: Ejecución de Agentes en Contenedores Aislados con COSMOS"
description = """Fecha: 2026-02"""
lang = "es"
category = "design"
subcategory = "core"
+++

# ADR-005: Ejecución de Agentes en Contenedores Aislados con COSMOS

**Fecha**: 2026-02
**Estado**: Aceptado

## Contexto

En un sistema multi-agente donde los agentes ejecutan código generado por LLM, el aislamiento entre agentes es crítico para:

1. **Seguridad**: La salida no confiable del LLM no debe poder acceder a la memoria, archivos o conexiones de red de otro agente.
1. **Aislamiento de estado**: El estado REPL de cada agente (variables JavaScript, bindings, snapshots) debe ser independiente.
1. **Control de recursos**: Un agente que se comporta mal no debe consumir CPU, memoria o PIDs ilimitados.
1. **Reproducibilidad**: El estado del agente debe ser capturable y restaurable para depuración y reversión.
1. **Flujos de trabajo fork/merge**: El sistema necesita soportar ramificación de ejecución de agentes (fork) y fusión de resultados (merge), similar a la ramificación de git.

Se evaluaron varios enfoques de aislamiento:

| Enfoque | Fuerza de Aislamiento | Control de Recursos | Snapshot/Fork | Sobrecarga |
| --- | --- | --- | --- | --- |
| **Contenedor por agente (Docker/OCI)** | Fuerte (nivel kernel) | Completo (cgroups, seccomp, capabilities) | Nativo (commit/snapshot) | Moderada (~100ms inicio, ~50MB por contenedor) |
| **Proceso por agente** | Moderado (UID/seccomp) | Parcial (rlimit) | Manual (serializar estado) | Baja |
| **Hilo por agente** | Débil (memoria compartida) | Mínimo | Manual | Mínima |
| **Sandbox WASM por agente** | Fuerte (memoria lineal) | Bueno (medición de gas) | Manual | Baja |
| **Contexto Boa por agente** | Moderado (sandbox JS) | Limitado | Integrado (serialización de espacio de nombres) | Mínima |

## Decisión

Elegimos una **arquitectura de contenedores de dos capas** con **COSMOS** como proceso init dentro del contenedor de cada agente:

**Capa externa (infraestructura de orquestación):**

- Docker/Podman vía Bollard para contenedores de infraestructura (PostgreSQL, demonio Scepter).
- Capacidades completas de orquestación: redes, volúmenes, health checks, compose.

**Capa interna (sandboxes de agente):**

- Youki/libcontainer (predeterminado) o Docker para contenedores COSMOS por agente.
- Cada agente obtiene su propio contenedor con COSMOS como PID 1.
- COSMOS es el **proceso frontal** que media todas las interacciones — proporciona el servidor Unix socket JSON-RPC, el REPL Boa JS, el router MCP y la conexión del puente HapLotes de vuelta a Scepter.

**Por qué COSMOS como intermediario obligatorio:**

Todas las interacciones con un agente contenedorizado deben pasar por COSMOS. La manipulación directa del contenedor (ej., `docker exec` en un contenedor) elude el modelo de seguridad, la gestión de estado y la pista de auditoría. COSMOS proporciona:

1. **Mediación de despacho de herramientas**: El `McpRouter` aplica listas de permitidos, doble autorización y niveles de confianza antes de que cualquier herramienta llegue al agente.
1. **Persistencia de estado**: El sistema de snapshot de doble búfer asegura que el estado REPL sobreviva a fallos.
1. **Comunicación puente**: El puente HapLotes conecta COSMOS de vuelta a Scepter para coordinación entre agentes.
1. **Aplicación de seguridad**: Los perfiles Seccomp, políticas de egreso y restricciones de capabilities se aplican en la creación del contenedor y son aplicadas por el kernel.

**Por qué Youki/libcontainer para sandboxes internos:**

- Sin root y sin demonio — no se requiere demonio Docker para sandboxes de agente.
- Compatible con OCI — especificación estándar `config.json`, compatible con herramientas OCI.
- Rootfs rápido basado en overlay — las operaciones de snapshot y fork copian solo archivos modificados.
- Menor sobrecarga que Docker para contenedores efímeros.

## Consecuencias

### Positivas

- **Aislamiento fuerte mediante aplicación del kernel**: cgroups (límites de CPU/memoria/PID), seccomp (filtrado de syscalls), capabilities (`cap_drop`=ALL), namespaces (aislamiento de PID/red/montaje).
- **Fork/merge nativo**: El commit de contenedor crea una imagen snapshot; se pueden crear nuevos contenedores desde el snapshot. Los sistemas de archivos overlay rastrean solo archivos modificados.
- **Límites de recursos por agente**: 512MB de memoria, 1 CPU, 100 PIDs por defecto, configurables por contenedor.
- **Pista de auditoría**: Todas las llamadas a herramientas pasan por el router MCP de COSMOS, que registra cada despacho para auditoría de seguridad OreXis.
- **Contención de fallos**: Un pánico de Boa o bug de agente está confinado a su contenedor. Otros agentes y Scepter continúan operando.
- **Youki para sandboxes ligeros**: Los contenedores internos inician más rápido y consumen menos recursos que los contenedores Docker completos.

### Negativas

- **Complejidad de COSMOS como PID 1**: COSMOS debe manejar reenvío de señales, recolección de zombies y apagado limpio como proceso init del contenedor. Esto añade responsabilidad que una aplicación normal no tiene.
- **Latencia de inicio de contenedor**: Cada contenedor de agente requiere ~100ms-1s para iniciar (dependiendo del runtime). Esto es más lento que el aislamiento basado en procesos o hilos.
- **Sobrecarga de recursos**: Cada contenedor COSMOS consume ~50-100MB de memoria para el runtime Boa, heap JS y sobrecarga del SO. Con 9 agentes contenedorizados, esto añade ~0.5-1GB de memoria base.
- **Complejidad de pruebas**: Probar el comportamiento del agente requiere ejecutar contenedores reales con COSMOS, lo que significa que las pruebas necesitan Docker o Youki disponible. El patrón de prueba snowflake (construir la imagen entelecheia, ejecutar un contenedor COSMOS, conectar vía Unix socket) es más complejo que las pruebas unitarias.
- **Dos runtimes que mantener**: Tanto las rutas de código Docker/Bollard como Youki/libcontainer deben mantenerse y probarse.

### Compromiso Aceptado

**Sobrecarga de recursos por garantías de seguridad y aislamiento.** Un modelo de proceso por agente usaría menos memoria e iniciaría más rápido, pero no proporcionaría aislamiento a nivel de kernel entre agentes. En un sistema donde los agentes ejecutan código generado por LLM no confiable, la garantía de seguridad del aislamiento por contenedor vale el costo de recursos. El diseño de COSMOS como intermediario obligatorio asegura que incluso si un atacante obtiene ejecución de código dentro de un contenedor, no puede eludir el modelo de seguridad operando fuera de la mediación de COSMOS.
