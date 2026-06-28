+++
title = "Arquitectura en Profundidad"
description = """> Audiencia: Desarrolladores que necesitan entender cómo funciona shittim-chest internamente."""
lang = "es"
category = "guides"
subcategory = "webui"
+++

# Arquitectura en Profundidad

> **Audiencia**: Desarrolladores que necesitan entender cómo funciona shittim-chest internamente.
> **Última actualización**: 2026-05-25

## Resumen del Proyecto

shittim-chest es la **interfaz de usuario** para [entelecheia](https://github.com/celestia-island/entelecheia), una plataforma de colaboración multi-agente basada en Rust. La frontera es deliberada:

- **entelecheia** posee la orquestación de agentes (scepter, 13 agentes, runtime Cosmos/IEPL), identidad y permisos.
- **shittim-chest** posee la autenticación de usuario, gestión de sesiones, datos de chat, configuración de proveedores LLM, presentación del frontend y el puente proxy hacia scepter.

Se comunican mediante HTTP y WebSocket autenticado con JWT. shittim-chest nunca accede directamente a la base de datos de entelecheia para operaciones de agentes.

## Stack del Backend

### Router Axum

El backend central (`packages/core`) es una aplicación Axum 0.8. El router monta estos grupos de módulos:

```text
/                   → health check
/api/auth/*         → AuthService (inicio de sesión, registro, GitHub OAuth, refresco, cierre de sesión)
/api/chat/*         → ChatService (conversaciones, mensajes, streaming SSE/WS, búsqueda, exportación)
/api/providers/*    → ProviderService (CRUD de proveedores LLM, cifrado de claves API, pruebas)
/api/generation/*   → GenerationService (generación de imágenes)
/api/devices/*      → DeviceService (listado de dispositivos remotos, sesiones, señalización)
/api/webhook/*      → WebhookService (GitHub, GitLab, Gitee, personalizado; validación HMAC)
/api/proxy/*        → ProxyService (proxy inverso HTTP + puente WebSocket a scepter)
/static/*           → Alojamiento estático SPA (solo producción)
```

### SeaORM + PostgreSQL

El acceso a la base de datos usa SeaORM 1.x con PostgreSQL. `shittim_chest_db` almacena:

- Autenticación de usuario: hashes de contraseñas (argon2), sesiones, tokens de refresco, claves API, conexiones OAuth
- Datos de chat: conversaciones, mensajes
- Configuraciones de proveedores LLM (claves API cifradas en reposo con AES-256-GCM)
- Registros de dispositivos remotos y sesiones de dispositivos
- Configuraciones de canales para mensajería multiplataforma
- Registros de entrega de webhooks

5 migraciones y 25 modelos de entidad residen en `packages/core/src/{migration,entity}/`.

### Autenticación JWT

`shittim_chest` emite JWT que contienen `{ sub: user_id, groups: [...] }`. El secreto JWT se comparte con scepter para que ambos servicios puedan validar tokens independientemente. Los tokens de acceso expiran en 1 hora; los tokens de refresco en 7 días con rotación en cada uso.

## Capacidad LLM Independiente

shittim-chest tiene su propia capa de enrutamiento LLM que opera independientemente de entelecheia:

- **LlmRouter**: Enrutador multi-proveedor con selección basada en prioridad y fallback
- **Gestión de proveedores**: Endpoints CRUD para añadir/editar/eliminar proveedores LLM
- **Cifrado de claves API**: Las claves API de los proveedores se cifran en reposo con AES-256-GCM
- **Compatible con OpenAI**: Funciona con cualquier API compatible con OpenAI (DeepSeek, OpenAI, modelos locales, etc.)
- **Streaming dual**: SSE (Server-Sent Events) y WebSocket streaming para respuestas de chat

Esto significa que shittim-chest puede ejecutarse como una aplicación de chat independiente sin entelecheia, o usar agentes de entelecheia mediante la capa proxy.

## Flujo de Autenticación

### Secuencia de Inicio de Sesión

```text
Usuario → shittim_chest: POST /api/auth/login { username, password }
shittim_chest → shittim_chest_db: SELECT user WHERE username = ? (verificar hash argon2)
shittim_chest → scepter: GET /api/user/{id}/permissions
scepter → entelecheia_db: consultar grupos + permisos
scepter → shittim_chest: { groups: [...], permissions: {...} }
shittim_chest → Usuario: { access_token, refresh_token }
shittim_chest: Almacenar sesión + cachear RBAC
```

### GitHub OAuth

```text
Usuario → shittim_chest: GET /api/auth/github
shittim_chest → Usuario: 302 redirigir a GitHub OAuth
Usuario → GitHub: autorizar
GitHub → shittim_chest: GET /api/auth/github/callback?code=...
shittim_chest → GitHub: intercambiar código por token de acceso
shittim_chest → GitHub: GET /user (obtener info del usuario)
shittim_chest → shittim_chest_db: INSERT/UPDATE oauth_connections
shittim_chest → Usuario: { access_token, refresh_token } (crea automáticamente el usuario si es nuevo)
```

## Arquitectura de Chat

### Flujo de Mensajes (LLM Independiente)

```text
Usuario → POST /api/chat/conversations/:id/messages
shittim_chest: validar JWT, cargar conversación
shittim_chest → LlmRouter: enrutar solicitud al mejor proveedor
LlmRouter → Proveedor LLM: POST chat/completions (streaming)
Proveedor LLM → LlmRouter: stream SSE
LlmRouter → Usuario: stream SSE/WS (tokens a medida que llegan)
shittim_chest: persistir mensaje en shittim_chest_db
```

### SSE vs WebSocket Streaming

- **SSE** (`/api/chat/stream`): Streaming HTTP simple, funciona a través de proxies, reconexión automática
- **WebSocket** (`/ws/chat/stream`): Bidireccional, soporta cancelación e interacción en tiempo real

## Arquitectura Proxy

El endpoint `/api/proxy/*` reenvía solicitudes autenticadas a scepter:

1. El navegador abre `ws://shittim-chest:80/api/proxy/chat` con JWT
1. `shittim_chest` valida el JWT, abre conexión a scepter reenviando el JWT
1. Reenvío bidireccional de mensajes entre el navegador y scepter
1. `shittim_chest` aplica límites de tasa, registra el uso, gestiona el ciclo de vida de la conexión

## Pipeline de Webhooks

Los webhooks de servicios externos entran a través de `/api/webhook/*`:

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → Validación HMAC → Parsear evento → Reenviar a scepter mediante socket Unix
```

Fuentes soportadas: GitHub (HMAC-SHA256), GitLab (token), Gitee (HMAC + fallback de token), más un endpoint genérico `/api/webhook/custom/{name}`. Características:

- Detección de entregas duplicadas (caché LRU, 10,000 IDs)
- Registro de entregas con API de listado
- Lista blanca de IPs para fuentes de webhook

## Gestión de Dispositivos Remotos

Los dispositivos remotos se gestionan a través de un relay de señalización:

```text
Navegador (webui) → WS /api/devices/stream → shittim_chest (relay de señal) → Socket Unix → entelecheia/polemos
```

Características:

- Listado de dispositivos y CRUD de sesiones mediante REST
- Señalización WebRTC (oferta/respuesta SDP, candidatos ICE)
- Relay de terminal (WebSocket a xterm.js)
- Relay de frames de escritorio
- Backend de explorador de archivos SFTP

shittim-chest nunca se conecta directamente a dispositivos remotos — todos los datos fluyen a través del agente polemos de entelecheia.

## Propiedad de los Datos

### shittim_chest_db

| Datos | Tabla | Justificación |
| --- | --- | --- |
| Hashes de contraseñas (argon2) | `auth_users` | La capa de presentación posee el flujo de inicio de sesión |
| Sesiones activas, tokens de refresco | `sessions` | La gestión de sesiones es una preocupación del frontend |
| Claves API cifradas | `api_keys` | La emisión de claves API es orientada al usuario |
| Conexiones OAuth | `oauth_connections` | La vinculación de autenticación de terceros es orientada al usuario |
| Conversaciones, mensajes | `conversations`, `messages` | Los datos de chat son orientados al usuario |
| Configuraciones de proveedores LLM | `llm_providers` | La gestión de proveedores es orientada al usuario (claves cifradas) |
| Registros de dispositivos remotos | `remote_devices`, `device_sessions` | El seguimiento de dispositivos es orientado al usuario |
| Configuraciones de canales | `channel_configs`, etc. | La configuración multiplataforma es orientada al usuario |

### entelecheia_db

| Datos | Justificación |
| --- | --- |
| Identidad de usuario, grupos, asignaciones de roles | El núcleo aplica los permisos |
| GroupPermissions (cuotas de proveedor, listas blancas de agentes) | La política a nivel de agente reside con los agentes |
| Configuraciones de agentes, estado Cosmos/IEPL | Los datos de orquestación pertenecen al núcleo |

## Estrategia de Frontend Dual

### Fase 1: Vue 3 (Actual)

| Paquete | Tecnología | Puerto | Propósito |
| --- | --- | --- | --- |
| `webui` | Vue 3 + Vite + Pinia (TSX) | `:3000 (compartido)` | Webui unificado: chat, generación de imágenes, dispositivos, admin (proveedores, agentes, RBAC, webhooks) |

### Fase 2: Rust WASM (Futuro)

| Paquete | Tecnología | Propósito |
| --- | --- | --- |
| `webui` | Rust → WASM (Tairitsu) | Webui unificado a largo plazo (chat + admin) |

Los frontends heredados sirven como especificaciones vivas. Durante la transición, ambas versiones se ejecutan en paralelo, e interacciones de usuario idénticas deben producir resultados idénticos.

## Modos de Despliegue con Proxy Inverso

shittim-chest soporta tres modos de proxy inverso, controlados por `SHITTIM_CHEST_PROXY_MODE` en `.env`.

### Modo 1: Ninguno (Directo)

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=none   # o sin configurar
```

El servidor central se vincula directamente a `SHITTIM_CHEST_HOST:SHITTIM_CHEST_PORT` (por defecto `0.0.0.0:80`). Sin TLS, sin contenedor de proxy inverso. Adecuado para:

- Desarrollo local
- Detrás de un proxy inverso existente (Cloudflare Tunnel, AWS ALB, etiquetas Traefik)
- Redes Docker donde otro servicio maneja la terminación TLS

### Modo 2: Caddy Auto

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_DOMAIN=app.ejemplo.com
```

El CLI crea un contenedor `shittim-chest-caddy` (imagen `caddy:2`) que:

1. Escucha en los puertos 80/443 (configurable mediante `SHITTIM_CHEST_PROXY_HTTP_PORT` / `SHITTIM_CHEST_PROXY_HTTPS_PORT`)
1. Aprovisiona automáticamente certificados TLS mediante Let's Encrypt (ACME integrado de Caddy)
1. Proxy de todas las solicitudes al backend central en la red Docker

No se necesita Caddyfile — el CLI genera uno automáticamente. El dominio debe tener DNS público apuntando al host.

### Modo 3: Caddy Personalizado

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/caddy/Caddyfile
SHITTIM_CHEST_PROXY_EXTRA_VOLUMES=/etc/letsencrypt:/etc/letsencrypt
```

Mismo contenedor Caddy, pero proporcionas tu propio Caddyfile (montado desde el host). Úsalo cuando necesites:

- Múltiples hosts virtuales
- Rutas de certificados TLS personalizadas
- Middleware adicional (autenticación básica, limitación de tasa, etc.)
- Servir archivos estáticos junto a la API

### Modo 4: Nginx Personalizado

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=nginx
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/nginx/conf.d/default.conf
```

Crea un contenedor `nginx:bookworm` con tu archivo de configuración. Tú gestionas los certificados TLS. Adecuado para entornos donde Nginx es el estándar.

### Ciclo de Vida del Contenedor

Todos los contenedores proxy son gestionados por el CLI mediante la API Docker (`bollard`):

| Comando | Comportamiento |
| --- | --- |
| `just dev` / `chest up` | Crea/inicia el contenedor proxy si `PROXY_MODE` está configurado |
| `just dev-stop` / `chest down` | Detiene y elimina el contenedor proxy |
| Contenedor ya en ejecución | Reutiliza el contenedor existente (idempotente) |

El contenedor proxy se une a la misma red Docker que el backend central, por lo que alcanza el backend mediante el nombre de host interno (`core` o `shittim-chest`).
