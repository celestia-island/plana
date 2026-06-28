+++
title = "Arquitectura de Seguridad de Entelecheia"
description = """> Modelo integral de defensa en profundidad para la Plataforma de Orquestación Multi-Agente Entelecheia."""
lang = "es"
category = "design"
subcategory = "core"
+++

# Arquitectura de Seguridad de Entelecheia

> Modelo integral de defensa en profundidad para la Plataforma de Orquestación Multi-Agente Entelecheia.

## Descripción General

Entelecheia implementa una **arquitectura de seguridad de defensa en profundidad** que abarca 14 capas de seguridad independientemente comprobables — desde el aislamiento de contenedores a nivel de hardware hasta las puertas de permiso de herramientas orientadas al LLM. A diferencia de los frameworks de agentes tradicionales que exponen todas las herramientas directamente al LLM, el diseño de **Microkernel Solo-Ejecución** de Entelecheia significa que el LLM ve solo 3 herramientas primitivas (`exec`, `write_to_var`, `write_to_var_json`), mientras que 148 herramientas MCP se despachan a través de un pipeline IEPL tipado con autorización multicapa.

## Índice de Capas de Seguridad

| # | Capa | Crate(s) | Amenaza Mitigada |
| --- | --- | --- | --- |
| 1 | Microkernel Solo-Ejecución | `scepter`, `mcp_types` | Acceso irrestricto a herramientas por el LLM |
| 2 | Puerta de Permiso de Doble Autorización | `security_policy` | Invocación no autorizada de herramientas MCP |
| 3 | Autorización de Habilidades por Nivel de Confianza | `domain_skills_permissions` | Escalada de privilegios mediante encadenamiento de habilidades |
| 4 | Aislamiento de Contenedor (Externo) | `container` (Docker/Podman) | Compromiso del host desde código de agente |
| 5 | Sandbox OCI (Interno) | `container_runtime` (Youki/libcontainer) | Escape de contenedor |
| 6 | Control de Acceso RBAC | `domain_auth`, shittim-chest `rbac` | Acceso no autorizado a API |
| 7 | Autenticación JWT | shittim-chest `auth` (HS256) | Secuestro de sesión, ataques de repetición |
| 8 | Encriptación de Claves API | `aporia` (AES-256-GCM) | Fuga de credenciales en reposo |
| 9 | Centinela de Seguridad | `orexis` (agente OreXis) | Ejecución de código malicioso, violaciones de cumplimiento |
| 10 | Pipeline IEPL con Seguridad de Tipos | `iepl`, `iepl_engine`, `skemma` | Inyección mediante llamadas a herramientas sin tipo |
| 11 | Lista Blanca de Registro de Proveedores | `config/registries.toml` | Ataques a la cadena de suministro mediante paquetes no confiables |
| 12 | Defensa contra Inyección de Prompts | Límite del sandbox IEPL | Inyección de prompts LLM mediante salida de herramientas |
| 13 | Limitación de Tasa | shittim-chest `channel/rate_limit` | DoS, agotamiento de recursos |
| 14 | Pista de Auditoría | `orexis`, `timeline` | Forense post-incidente, responsabilidad |

---

## Capa 1: Microkernel Solo-Ejecución

**Crates:** `scepter`, `mcp_types`
**Filosofía de Diseño:** Minimizar la superficie de ataque del LLM

El LLM opera en un **sandbox solo-ejecución** donde puede invocar solo tres operaciones primitivas:

| Herramienta | Propósito | Parámetros |
| --- | --- | --- |
| `exec` | Ejecutar una cadena de script | Código JavaScript (transpilado desde TypeScript vía IEPL) |
| `write_to_var` | Almacenar un valor de cadena | Nombre de variable + valor |
| `write_to_var_json` | Almacenar un valor JSON | Nombre de variable + valor JSON |

Las 148 herramientas MCP (operaciones de archivos, gestión de contenedores, control de dispositivos, búsqueda web, etc.) son **invisibles para el LLM**. Se invocan indirectamente a través del pipeline IEPL cuando el `exec` del LLM llama a importaciones de módulos ES (ej., `import { file_read } from 'kalos'`).

**Modelo de amenaza:** Incluso si el LLM se ve comprometido mediante inyección de prompts, no puede invocar directamente herramientas peligrosas como `container_destroy` o `ssh_exec`. El pipeline IEPL aplica verificación de tipos y verificación de permisos antes de que cualquier herramienta se ejecute.

**Implementación:** `packages/shared/mcp_types/src/` define los tipos IPC del microkernel. El manejador `exec` en `packages/cosmos/` transpila y ejecuta el script mediante el motor Boa, con llamadas a herramientas enrutadas a través del `McpRouter` de `skemma`.

---

## Capa 2: Puerta de Permiso de Doble Autorización

**Crate:** `security_policy` (5,772 líneas)

Cada herramienta MCP declara sus requisitos de acceso mediante un enum de **nivel de permiso**. Cada habilidad (script IEPL) declara el nivel de permiso que necesita por herramienta. Ambos deben coincidir para que una llamada proceda.

```rust
pub enum PermissionLevel {
    /// Operaciones de solo lectura (file_read, list_dir, etc.)
    Read,
    /// Operaciones de escritura dentro del espacio de trabajo (file_write, exec_script)
    Write,
    /// Operaciones que afectan sistemas externos (ssh_exec, container_deploy)
    System,
    /// Operaciones con consecuencias irreversibles (container_destroy, device_reboot)
    Destructive,
}
```

**Flujo de autorización:**

1. La habilidad declara: "Necesito acceso `System` a `ssh_exec`"
1. La herramienta declara: "Requiero permiso `System`"
1. La puerta de permiso verifica: `skill_level >= tool_requirement` Y `la habilidad tiene concedida explícitamente esta herramienta`
1. Si alguna verificación falla: la llamada se bloquea, se registra y se reporta al centinela OreXis

**Implementación:** `packages/shared/security_policy/src/` — 107 anotaciones de prueba, 4 pruebas tokio.

---

## Capa 3: Autorización de Habilidades por Nivel de Confianza

**Crate:** `domain_skills_permissions` (1,776 líneas)

Las habilidades se clasifican en **niveles de confianza** que determinan su alcance de permiso predeterminado:

| Nivel de Confianza | Descripción | Permisos Predeterminados |
| --- | --- | --- |
| `Builtin` | Se distribuye con la plataforma | Acceso completo a herramientas |
| `Verified` | Revisado y firmado por mantenedores | Lectura + Escritura |
| `Community` | Enviado por usuarios | Solo lectura |
| `Untrusted` | Cargado dinámicamente | Sin acceso a herramientas (solo exec) |

El nivel de confianza de cada habilidad se verifica al cargar y se almacena en caché. Los intentos de escalar el nivel de confianza se registran como eventos de seguridad.

---

## Capa 4: Aislamiento de Contenedor (Anillo Externo)

**Crate:** `container` (5,742 líneas)

Cada ejecución de agente ocurre dentro de un **contenedor Docker o Podman** con:

- Aislamiento de espacio de nombres de red
- Sistema de archivos raíz de solo lectura (excepto montaje de workspace)
- Perfil Seccomp que restringe syscalls
- Límites de recursos (CPU, memoria, conteo de PID)
- Sin acceso al socket Docker del host

**Implementación:** `packages/shared/container/src/` — 74 anotaciones de prueba, 12 pruebas tokio. Soporta tanto Docker (vía API Bollard) como Podman.

---

## Capa 5: Sandbox OCI (Anillo Interno)

**Crate:** `container_runtime` (3,645 líneas)

Dentro del contenedor Docker, Entelecheia ejecuta una **segunda capa de aislamiento** usando Youki/libcontainer — un runtime de contenedor compatible con OCI, sin demonio y sin root. Esto proporciona:

- Ejecución sin root (sin posibilidad de escalada de privilegios)
- Aislamiento de espacio de nombres independiente de Docker
- Aplicación de Cgroup v2
- Filtro Seccomp (denegar por defecto)

**¿Por qué dos capas?** Docker proporciona aislamiento de grano grueso (red, sistema de archivos). Youki proporciona filtrado de syscalls de grano fino y contabilidad de recursos. Si Docker se ve comprometido, el sandbox Youki aún contiene al agente.

---

## Capa 6: Control de Acceso RBAC

**Crates:** `domain_auth` (380 líneas), shittim-chest `rbac` (1,736 líneas)

Control de acceso basado en roles que gobierna todas las operaciones API:

- **Grupos:** Los usuarios pertenecen a grupos; los grupos tienen concesiones
- **Concesiones:** Permisos de grano fino (lectura/escritura/admin por tipo de recurso)
- **Aislamiento de workspace:** Los usuarios solo pueden acceder a los workspaces de los que son miembros
- **Operaciones entre workspaces:** Requieren concesiones de admin explícitas

---

## Capa 7: Autenticación JWT

**Módulo:** shittim-chest `auth/jwt.rs` (264 líneas)

- **Algoritmo:** HS256 (HMAC-SHA256)
- **Tokens de acceso:** Vida corta (configurable, predeterminado 15 min)
- **Tokens de actualización:** Vida más larga con rotación al usar
- **Protección CSRF basada en nonce** para clientes de navegador
- **Limitación de tasa** en endpoints de autenticación (algoritmo GCRA)

---

## Capa 8: Encriptación de Claves API

**Crate:** `aporia` (5,802 líneas)

Todas las claves API de proveedores LLM se encriptan en reposo usando **AES-256-GCM** con:

- Nonce único por operación de encriptación
- Clave derivada de un secreto maestro (configurado por entorno)
- Puesta a cero de claves en texto plano de la memoria después del uso
- Soporte de rotación de claves

---

## Capa 9: Centinela de Seguridad (OreXis)

**Crate:** `orexis` (5,239 líneas) — el agente "sistema inmunológico"

OreXis es un Agente de Capa-1 que:

- **Audita código** en busca de vulnerabilidades de seguridad y cumplimiento de licencias
- **Inspecciona llamadas a herramientas** contra políticas de seguridad registradas
- **Bloquea/desbloquea** las herramientas de cualquier agente por patrón
- **Monitorea** el comportamiento del agente en busca de patrones anómalos

Herramientas MCP (24): `standard_check`, `compliance_report`, `audit_alignment`, `audit_legality`, `agent_integrity`, `security_audit`, `tool_block`, `tool_unblock`, `policy_register`, `policy_list`, etc.

---

## Capa 10: Pipeline IEPL con Seguridad de Tipos

**Crates:** `iepl` (2,670 líneas), `iepl_engine` (1,228 líneas), `skemma` (7,960 líneas)

El pipeline **Lenguaje de Plugin de Entelecheia** (IEPL) asegura la seguridad de tipos entre el código generado por LLM y el despacho de herramientas nativas:

1. El LLM genera código TypeScript usando importaciones de módulos ES
1. **SWC** transpila TypeScript → JavaScript (validación de sintaxis)
1. El **motor Boa** ejecuta JavaScript en un contexto aislado
1. Las importaciones de módulos ES se resuelven a llamadas `__native_dispatch`
1. Cada despacho se enruta a través de `McpRouter` con verificación completa de tipos

**Amenaza mitigada:** Ataques de inyección mediante llamadas a herramientas sin tipo (común en frameworks de agentes basados en Python donde los esquemas de herramientas se validan solo en tiempo de ejecución).

---

## Capa 11: Lista Blanca de Registro de Proveedores

**Archivo:** `configs/registries.toml` (337 líneas)

Entelecheia mantiene una **lista blanca hardcodeada** de registros de paquetes confiables en 15 ecosistemas:

crates.io, PyPI, npm, Go modules, Docker Hub, Maven Central, NuGet, RubyGems, Hackage, Alpine APK, Debian APT, GitHub, GitLab, `HuggingFace`, PyTorch.

Cualquier importación de paquete desde un registro no incluido en la lista blanca se **bloquea a nivel de contenedor** antes de la ejecución.

---

## Capa 12: Defensa contra Inyección de Prompts

**Mecanismo:** Límite del sandbox IEPL

La salida `exec` del LLM se ejecuta en un **contexto Boa JS aislado** sin acceso a:

- El sistema de archivos del host
- Sockets de red
- Variables de entorno
- El estado de otros agentes

Las salidas de herramientas devueltas al LLM son **sanitizadas** — los datos binarios se codifican en base64, la salida excesiva se trunca y los patrones potenciales de inyección de prompts en los resultados de herramientas son marcados por OreXis.

---

## Capa 13: Limitación de Tasa

**Módulo:** shittim-chest `channel/rate_limit.rs` (118 líneas)

Limitación de tasa por usuario y por canal usando el **Algoritmo GCRA (Generic Cell Rate Algorithm)**:

- Tamaño de ráfaga y tasa sostenida configurables
- DashMap por usuario para búsqueda O(1)
- Retroceso automático al exceder el límite
- Límites separados para llamadas API, envíos de mensajes e invocaciones de herramientas

---

## Capa 14: Pista de Auditoría

**Crates:** `orexis`, `timeline` (3,096 líneas)

Cada invocación de herramienta, decisión de agente y evento de seguridad es:

1. Registrado en la **línea de tiempo** con contexto completo (insignia de agente, nombre de habilidad, parámetros, resultado)
1. Enlazado por hash a eventos anteriores para detección de manipulación
1. Persistido en PostgreSQL con retención configurable
1. Consultable mediante CLI (`entelecheia-cli trace-chain <insignia>`)

---

## Comparación de Seguridad con Otros Frameworks

| Característica | Entelecheia | OpenFANG | LangChain | Claude Code |
| --- |  ---  |  ---  |  ---  |  ---  |
| Herramientas visibles al LLM | **3 (solo-exec)** | 53 (todas visibles) | Todas visibles | 33 (todas visibles) |
| Aislamiento de contenedor | **Doble capa** (Docker + Youki) | Solo WASM | Ninguno | Nivel SO (Seatbelt/Landlock) |
| Modelo de permiso de herramientas | **Doble autorización** | RBAC | Ninguno | Ninguno |
| Agente de auditoría de código | **OreXis (24 herramientas)** | Guardia de bucle | Ninguno | Ninguno |
| Despacho con seguridad de tipos | **Pipeline IEPL** | Llamada directa a función | Llamada directa a función | Llamada directa a función |
| Lista blanca de paquetes | **15 registros** | Ninguno | Ninguno | Ninguno |
| Pista de auditoría | Línea de tiempo enlazada por hash | Cadena de hash Merkle | Ninguno | Ninguno |

---

## Modelo de Amenazas

### Fuera de Alcance

- Acceso físico a máquinas host
- Demonio Docker/Podman comprometido (se asume confiable)
- Exploits de kernel (mitigados pero no prevenidos por aislamiento en espacio de usuario)
- Ataques a la cadena de suministro en dependencias de crates Rust (parcialmente mitigados por `cargo-deny`)

### Riesgos Aceptados

- Vulnerabilidades del motor Boa JS (aislado dentro del contenedor)
- Interrupciones del proveedor LLM (sin ruta de ejecución de respaldo)
- Corrupción de datos PostgreSQL (mitigada por respaldos, no prevenida)

---

## Reporte de Vulnerabilidades

Consulte [SECURITY.md](../SECURITY.md) para el proceso de reporte de vulnerabilidades.

## Licencia

Esta arquitectura de seguridad es parte de Entelecheia, licenciada bajo [BUSL-1.1](../LICENSE).
