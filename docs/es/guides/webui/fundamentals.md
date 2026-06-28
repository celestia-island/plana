+++
title = "Conceptos Fundamentales"
description = """> Audiencia: Desarrolladores que quieren una comprensión conceptual del diseño de shittim-chest."""
lang = "es"
category = "guides"
subcategory = "webui"
+++

# Conceptos Fundamentales

> **Audiencia**: Desarrolladores que quieren una comprensión conceptual del diseño de shittim-chest.
> **Última actualización**: 2026-05-25

## Arquitectura de Dos Repositorios

shittim-chest y [entelecheia](https://github.com/celestia-island/entelecheia) forman un sistema de dos repositorios con una frontera deliberada:

- **entelecheia** — núcleo de orquestación de agentes (scepter, 13 agentes, runtime Cosmos/IEPL). Posee identidad, permisos, configuraciones de agentes.
- **shittim-chest** — interfaz de usuario. Posee autenticación, sesiones, datos de chat, configuración de proveedores LLM, UI del frontend y el puente proxy hacia scepter.

Se comunican mediante HTTP y WebSocket autenticado con JWT. Ninguno accede directamente a la base de datos del otro. Esta separación permite que cada repositorio se desarrolle, despliegue y escale independientemente.

## Modos de Operación Dual

shittim-chest soporta dos modos de operación:

### Modo Independiente

Se ejecuta de forma independiente con su propia capa de enrutamiento LLM. Soporta:

- Chat con respuestas en streaming (SSE + WebSocket)
- Generación de imágenes mediante proveedores configurados
- Autenticación de usuario (contraseña + GitHub OAuth)
- Gestión de proveedores (añadir/eliminar proveedores LLM)

No requiere entelecheia. Útil para desarrollo y despliegues simples.

### Modo Proxy

Actúa como puerta de enlace al sistema de agentes de entelecheia. Añade:

- Reenvío de solicitudes a scepter con paso de JWT
- Puenteo WebSocket para chat basado en agentes
- Ingreso de webhooks y reenvío de disparadores
- Gestión de dispositivos remotos mediante polemos
- Consultas de permisos RBAC y caché

Requiere una instancia de entelecheia en ejecución. Los dos modos pueden coexistir — LLM independiente para chat simple, proxy para orquestación de agentes.

## Modelo de Autenticación

La autenticación usa tokens JWT emitidos por `shittim_chest`:

1. **Almacenamiento de credenciales**: Las contraseñas (hashes argon2), sesiones, tokens de refresco y claves API residen en `shittim_chest_db`.
1. **GitHub OAuth**: Los usuarios pueden iniciar sesión con GitHub; las cuentas se crean automáticamente en el primer inicio de sesión.
1. **Almacenamiento de permisos**: Los grupos de usuarios, roles y matrices de permisos residen en `entelecheia_db`.
1. **Flujo JWT**: Al iniciar sesión, `shittim_chest` verifica las credenciales localmente, luego obtiene los permisos de scepter. El JWT emitido contiene `{ sub: user_id, groups: [...] }`.
1. **Secreto compartido**: El secreto de firma JWT se comparte con scepter para que ambos servicios puedan validar tokens independientemente.
1. **Rotación de tokens**: Los tokens de acceso expiran en 1 hora; los tokens de refresco en 7 días. Los tokens de refresco se rotan en cada uso.

## Frontend (webui)

El webui es el frontend unificado en `packages/webui/`, con la interfaz de chat en `/` y el panel de administración en `/backend`, construido con Vue 3 + Vite + Pinia (TSX mediante `@vitejs/plugin-vue-jsx`).

## Sistema de Proveedores LLM

shittim-chest tiene una capa de enrutamiento LLM independiente:

- **Proveedores**: Endpoints de API LLM configurables (compatibles con OpenAI). Almacenados en `shittim_chest_db` con claves API cifradas con AES-256-GCM.
- **Enrutador**: Enrutamiento multi-proveedor con selección basada en prioridad y fallback automático.
- **Categorías**: Los proveedores pueden etiquetarse como `chat`, `image` o ambos.
- **Gestión**: CRUD completo mediante API REST y panel de administración webui. Los proveedores pueden probarse para conectividad.
- **Streaming**: Ambos protocolos SSE (simple, compatible con proxy) y WebSocket (bidireccional).

## Sistema de Chat

- **Conversaciones**: Sesiones de chat basadas en hilos con títulos y metadatos
- **Mensajes**: Soporta texto, imágenes y llamadas a herramientas (function calling)
- **Streaming**: Entrega de respuestas token por token en tiempo real mediante SSE o WebSocket
- **Búsqueda**: Búsqueda de texto completo en mensajes con consultas ILIKE
- **Exportación**: Las conversaciones pueden exportarse en formato JSON o Markdown
- **Generación de imágenes**: Generación de imágenes mediante prompts con proveedores configurados, con funcionalidad "Insertar en chat"

## Gestión de Dispositivos Remotos

shittim-chest proporciona una interfaz basada en navegador para dispositivos remotos gestionados por entelecheia/polemos:

- **Escritorio**: Visor de escritorio remoto basado en WebRTC con relay de frames
- **Terminal**: Emulador de terminal basado en xterm.js con relay WebSocket
- **Explorador de archivos**: Backend de explorador de archivos SFTP (esqueleto)
- **Señalización**: Relay de señalización WebRTC basado en WebSocket (oferta/respuesta SDP, candidatos ICE)

Toda la comunicación de dispositivos fluye a través del agente polemos de entelecheia — shittim-chest nunca se conecta directamente a los endpoints.

## Arquitectura Proxy

`shittim_chest` actúa como puerta de enlace entre los usuarios y scepter:

- **Proxy inverso HTTP**: `/api/proxy/*` reenvía solicitudes autenticadas a scepter con paso de JWT.
- **Puente WebSocket**: El streaming de chat usa reenvío WebSocket bidireccional (`navegador ↔ shittim_chest ↔ scepter`).

Esto permite que `shittim_chest` aplique límites de tasa, registre el uso y gestione el ciclo de vida de la conexión sin que scepter necesite manejar conexiones individuales del navegador.

## Pipeline de Webhooks

Los eventos externos llegan al núcleo de agentes a través de un pipeline de webhooks:

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → Validación HMAC → Parsear evento → Reenviar a scepter mediante socket Unix → Despacho de Agente
```

Cada proveedor tiene su propio mecanismo de validación:

- **GitHub**: HMAC-SHA256 mediante `X-Hub-Signature-256`
- **GitLab**: Token mediante `X-Gitlab-Token`
- **Gitee**: HMAC con fallback de token

Características adicionales: detección de entregas duplicadas (caché LRU), registro de entregas, lista blanca de IPs y un endpoint de webhook personalizado genérico.

## Modelo RBAC

Los permisos siguen un modelo RBAC basado en grupos:

- **Grupos**: Los usuarios pertenecen a uno o más grupos.
- **Roles**: Los grupos tienen roles asignados.
- **Permisos**: Cada rol define una matriz de permisos que cubre:
  - Cuotas de proveedor (tokens máximos, solicitudes máximas)
  - Listas blancas de agentes (a qué agentes puede acceder el grupo)
  - Capacidades administrativas (gestionar usuarios, configurar proveedores)

`shittim_chest` almacena en caché los permisos en proceso con un TTL (por defecto 5 minutos). La invalidación de caché ocurre al expirar el TTL, al cerrar sesión o ante cambios explícitos de permisos propagados desde scepter.

## Estrategia de Frontend

shittim-chest usa un enfoque de frontend en dos fases:

**Fase 1 (actual)**: Frontend Vue 3 (`webui`, en `packages/webui/`) construido con Vite + Pinia, usando TSX mediante `@vitejs/plugin-vue-jsx`. Define el contrato API y sirve como implementación de referencia de calidad de producción.

**Fase 2 (futuro)**: Frontend Rust → WASM construido con Tairitsu. El frontend heredado actúa como especificación viva y oráculo de pruebas — interacciones de usuario idénticas deben producir resultados idénticos.

## Puente de Seguridad de Tipos

Los tipos TypeScript se generan desde código Rust mediante la crate de protocolo externa `arona`, asegurando consistencia entre frontend y backend:

```text
arona Rust crate (dependencia git)
  → #[derive(ts_rs::TS)]
  → codegen ts-rs → packages/webui/src/types/arona/ (TypeScript)
  → consumido por webui como @celestia-island/arona
```

Esto elimina la sincronización manual de tipos. Cuando un tipo Rust en la crate `arona` cambia, los bindings TypeScript se regeneran y son consumidos por el webui.
