+++
title = "Guía de Construcción y Desarrollo"
description = """> Audiencia: Contribuyentes configurando un entorno de desarrollo local de shittim-chest."""
lang = "es"
category = "guides"
subcategory = "webui"
+++

# Guía de Construcción y Desarrollo

> **Audiencia**: Contribuyentes configurando un entorno de desarrollo local de shittim-chest.
> **Última actualización**: 2026-05-25

## Requisitos Previos

| Herramienta | Versión Mínima | Notas |
| --- | --- | --- |
| Rust | 1.85+ | Se requiere Edition 2024; instalar mediante <https://rustup.rs> |
| Node.js | 20+ | Se recomienda LTS |
| pnpm | 9+ | `corepack enable && corepack prepare pnpm@latest --activate` |
| just | latest | Ejecutor de comandos; `cargo install just` |
| PostgreSQL | 18+ | shittim_chest_db para autenticación + datos de chat |
| entelecheia scepter | opcional | Requerido para funciones de proxy/dispositivos; opcional para chat independiente |

Verifica todo:

```bash
rustc --version    # >= 1.85
node --version     # >= 20
pnpm --version     # >= 9
just --version
psql --version     # >= 18
```

## Clonar e Inicializar

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
```

## Variables de Entorno

Edita `.env` después de clonar. Cada variable está documentada en línea; a continuación un resumen.

### Servidor

| Variable | Por defecto | Propósito |
| --- | --- | --- |
| `SHITTIM_CHEST_HOST` | `0.0.0.0` | Dirección de escucha |
| `SHITTIM_CHEST_PORT` | `80` | Puerto de escucha |

### Base de Datos

| Variable | Por defecto | Propósito |
| --- | --- | --- |
| `SHITTIM_CHEST_DATABASE_URL` | `postgresql://sc:pass@localhost:5432/shittim_chest` | Cadena de conexión PostgreSQL |
| `SHITTIM_CHEST_DATABASE_MAX_CONNECTIONS` | `10` | Tamaño del pool de conexiones SeaORM |

Crea la base de datos y el usuario:

```sql
CREATE USER sc WITH PASSWORD 'pass';
CREATE DATABASE shittim_chest OWNER sc;
```

### JWT y Cifrado

| Variable | Por defecto | Propósito |
| --- | --- | --- |
| `JWT_SECRET` | `change-me-in-production` | Secreto compartido con scepter; **debe coincidir** |
| `JWT_EXPIRATION_SECONDS` | `3600` | Duración del token de acceso (1 hora) |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | Duración del token de refresco (7 días) |
| `SHITTIM_CHEST_ENCRYPTION_KEY` | `change-me-32-bytes-base64-encoded` | Clave AES-256-GCM para claves API y tokens OAuth |

Genera una clave de producción:

```bash
openssl rand -base64 32
```

### Proveedores LLM (para operación independiente)

Configúralos para usar shittim-chest independientemente sin entelecheia:

| Variable | Propósito |
| --- | --- |
| `LLM_DEFAULT_PROVIDER_ENDPOINT` | Endpoint de API compatible con OpenAI (ej. `https://api.deepseek.com/v1`) |
| `LLM_DEFAULT_PROVIDER_API_KEY` | Clave API para el proveedor |
| `LLM_DEFAULT_PROVIDER_MODELS` | Lista de modelos separada por comas (ej. `deepseek-chat,deepseek-reasoner`) |
| `LLM_DEFAULT_PROVIDER_CATEGORY` | Categoría del proveedor: `chat` o `image` |
| `LLM_STREAM_BUFFER_SECONDS` | Timeout del buffer de streaming (por defecto: 60) |
| `LLM_MAX_TOKENS_DEFAULT` | Tokens máximos por defecto (por defecto: 4096) |
| `LLM_REQUEST_TIMEOUT_SECONDS` | Timeout de solicitud HTTP (por defecto: 120) |

### Dispositivos Remotos

| Variable | Por defecto | Propósito |
| --- | --- | --- |
| `REMOTE_DEVICES_ENABLED` | `false` | Habilitar funciones de dispositivos remotos |
| `REMOTE_DEVICES_SCEPTER_SOCK` | `/run/entelecheia/device_stream.sock` | Socket Unix para datos de dispositivos |
| `REMOTE_DEVICES_FRAME_BUFFER_SIZE` | `4194304` | Tamaño del buffer de frames en bytes |
| `REMOTE_DEVICES_MAX_SESSIONS_PER_USER` | `3` | Máximo de sesiones de dispositivo concurrentes |
| `WEBRTC_ICE_SERVERS` | `stun:stun.l.google.com:19302` | Lista de servidores ICE |

### GitHub OAuth

| Variable | Propósito |
| --- | --- |
| `GITHUB_CLIENT_ID` | Client ID de la OAuth App de GitHub |
| `GITHUB_CLIENT_SECRET` | Client Secret de la OAuth App de GitHub |
| `GITHUB_REDIRECT_URI` | URL de callback OAuth (ej. `https://tu-dominio/api/auth/github/callback`) |

### Conexión Scepter (para funciones proxy)

| Variable | Por defecto | Propósito |
| --- | --- | --- |
| `ENTELECHEIA_SCEPTER_URL` | `http://localhost:8424` | Endpoint HTTP para scepter |
| `ENTELECHEIA_SCEPTER_WS_URL` | `ws://localhost:8424` | Endpoint WebSocket para scepter |
| `ENTELECHEIA_TUI_SOCK` | `/run/entelecheia/entelecheia.sock` | Socket Unix para reenvío de disparadores |

### Webhook

| Variable | Propósito |
| --- | --- |
| `WEBHOOK_GITHUB_SECRET` | Secreto HMAC para validación de webhooks de GitHub |
| `WEBHOOK_GITLAB_SECRET` | Token para validación de webhooks de GitLab |
| `WEBHOOK_PUBLIC_URL` | URL pública para endpoints de webhook |

## Configuración de la Base de Datos

```bash
just db-init      # Crear esquema (ejecuta migraciones SeaORM)
just db-migrate   # Aplicar migraciones pendientes
```

### Resumen del Esquema

`shittim_chest_db` posee los datos orientados al usuario:

| Tabla | Propósito |
| --- | --- |
| `auth_users` | Cuentas de usuario con hashes de contraseña argon2 |
| `sessions` | Sesiones activas con tokens de refresco |
| `api_keys` | Registros de claves API (hasheadas) |
| `oauth_connections` | Vinculaciones OAuth de terceros (GitHub) |
| `conversations` | Conversaciones de chat |
| `messages` | Mensajes de chat con datos de llamadas a herramientas |
| `llm_providers` | Configuraciones de proveedores LLM (claves API cifradas) |
| `remote_devices` | Registros de dispositivos remotos |
| `device_sessions` | Sesiones de dispositivo activas |
| `channel_configs` | Configuraciones de canales multiplataforma |
| `channel_messages` | Registros de mensajes de canal |
| `channel_pairings` | Emparejamientos canal-a-chat |

Reiniciar la base de datos:

```bash
just db-reset
```

## Desarrollo del Backend

```bash
just dev-backend
```

Esto ejecuta `cargo run --package shittim_chest`. El servidor inicia en `:80`.

### Comandos CLI

```bash
shittim_chest db-init      # Crear esquema de base de datos
shittim_chest db-migrate   # Aplicar migraciones pendientes
shittim_chest db-reset     # Eliminar y recrear esquema
shittim_chest server       # Iniciar el servidor web
```

### Hot Reload

```bash
cargo install cargo-watch
cargo watch -x 'run --package shittim_chest -- server'
```

### Resumen de Endpoints API

| Grupo de Rutas | Propósito |
| --- | --- |
| `/api/auth/*` | Inicio de sesión, registro, GitHub OAuth, refresco, cierre de sesión |
| `/api/chat/*` | Conversaciones, mensajes, streaming SSE/WS, búsqueda, exportación |
| `/api/providers/*` | CRUD de proveedores LLM, gestión de claves API, pruebas |
| `/api/generation/*` | Generación de imágenes, listado de modelos |
| `/api/devices/*` | Listado de dispositivos remotos, sesiones, señalización WebRTC |
| `/api/webhook/*` | Ingreso de webhooks GitHub/GitLab/Gitee/personalizado |
| `/api/proxy/*` | Proxy inverso a scepter (HTTP + WebSocket) |
| `/static/*` | Alojamiento de archivos estáticos SPA |

## Desarrollo del Frontend

### Instalar Dependencias

```bash
pnpm install
```

### webui

```bash
just dev    # construir frontend + iniciar backend en :3000
just watch  # reconstrucción automática ante cambios de archivos
```

Ambos frontends son construidos por Vite en `dist/`. El backend sirve estos archivos estáticos directamente en `:3000` — no se necesita un servidor dev Vite separado ni proxy. En modo dev, `dev.py` vigila las fuentes del frontend y reconstruye automáticamente.

## Configuración Multi-Proyecto

Para desarrollo local con la crate de protocolo compartida `arona`, aplícale un parche a tu checkout local. Edita `~/.cargo/config.toml` (nunca se commitea al repositorio):

```toml
[patch.'https://github.com/celestia-island/arona']
arona = { path = "/ruta/a/arona" }
```

Para npm, el webui consume los bindings TS de la crate `arona` mediante el alias de ruta `@celestia-island/arona`, apuntando a `packages/webui/src/types/arona/`.

## Construcción para Producción

```bash
just build
```

Esto ejecuta `cargo build --release` y `pnpm run build:all`. Ubicaciones de salida:

- Binario del backend: `target/release/shittim_chest`
- Activos del frontend: `packages/webui/dist/`

### Docker

Construye y ejecuta con el wrapper CLI (usa la API Docker directamente):

```bash
just dev
```

O manualmente:

```bash
just build        # construir imagen Docker
just up           # iniciar todos los servicios
just migrate      # ejecutar migraciones de base de datos
```

El binario de producción sirve los activos del frontend mediante el middleware de archivos estáticos de Axum en `/`. No se necesita un servidor de frontend separado.

## Problemas Comunes

### Conexión a la base de datos rechazada

```text
error: connection to server at "localhost", port 5432 failed
```

**Solución**: Asegúrate de que PostgreSQL esté en ejecución y `SHITTIM_CHEST_DATABASE_URL` en `.env` coincida con tu configuración. Verifica con `psql "$SHITTIM_CHEST_DATABASE_URL" -c 'SELECT 1'`.

### Scepter no alcanzable

```text
error: error sending request for url (http://localhost:8424/...)
```

**Solución**: Inicia la instancia scepter de entelecheia, o usa el modo independiente con proveedores LLM configurados. El backend funciona sin scepter para chat/generación de imágenes.

### Errores CORS en el navegador

```text
Access-Control-Allow-Origin header is present on the requested resource
```

**Solución**: El backend de desarrollo habilita CORS para orígenes `localhost`. Si cambiaste los puertos, actualiza la configuración CORS. Los despliegues de producción deben configurar un proxy inverso (nginx/caddy) para manejar CORS.

### pnpm install falla

**Solución**: Asegúrate de estar usando pnpm 9+. Ejecuta `corepack enable && corepack prepare pnpm@latest --activate` para configurar la versión correcta.

### cargo build falla en crates compartidas

**Solución**: Si tienes parches locales en `~/.cargo/config.toml`, asegúrate de que las rutas existan y los nombres de crate coincidan. Elimina la sección de parche para usar dependencias git en su lugar.
