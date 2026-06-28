+++
title = "Architecture Deep Dive"
description = """> Audience: Developers who need to understand how shittim-chest works internally."""
lang = "en"
category = "guides"
subcategory = "webui"
+++

# Architecture Deep Dive

> **Audience**: Developers who need to understand how shittim-chest works internally.
> **Last updated**: 2026-05-25

## Project Overview

shittim-chest is the **user-facing shell** for [entelecheia](https://github.com/celestia-island/entelecheia), a Rust-based multi-agent collaboration platform. The boundary is deliberate:

- **entelecheia** owns agent orchestration (scepter, 13 agents, Cosmos/IEPL runtime), identity, and permissions.
- **shittim-chest** owns user authentication, session management, chat data, LLM provider configuration, frontend presentation, and the proxy bridge to scepter.

They communicate via JWT-authenticated HTTP and WebSocket. shittim-chest never directly accesses entelecheia's database for agent operations.

## Backend Stack

### Axum Router

The core backend (`packages/core`) is an Axum 0.8 application. The router mounts these module groups:

```text
/                   → health check
/api/auth/*         → AuthService (login, register, GitHub OAuth, refresh, logout)
/api/chat/*         → ChatService (conversations, messages, SSE/WS streaming, search, export)
/api/providers/*    → ProviderService (LLM provider CRUD, API key encryption, testing)
/api/generation/*   → GenerationService (image generation)
/api/devices/*      → DeviceService (remote device listing, sessions, signaling)
/api/webhook/*      → WebhookService (GitHub, GitLab, Gitee, custom; HMAC validation)
/api/proxy/*        → ProxyService (HTTP reverse proxy + WebSocket bridge to scepter)
/static/*           → SPA static hosting (production only)
```

### SeaORM + PostgreSQL

Database access uses SeaORM 1.x with PostgreSQL. The `shittim_chest_db` stores:

- User authentication: password hashes (argon2), sessions, refresh tokens, API keys, OAuth connections
- Chat data: conversations, messages
- LLM provider configurations (API keys encrypted at rest with AES-256-GCM)
- Remote device records and device sessions
- Channel configurations for multi-platform messaging
- Webhook delivery logs

5 migrations and 25 entity models live in `packages/core/src/{migration,entity}/`.

### JWT Authentication

`shittim_chest` issues JWTs containing `{ sub: user_id, groups: [...] }`. The JWT secret is shared with scepter so both services can validate tokens independently. Access tokens expire in 1 hour; refresh tokens in 7 days with rotation on each use.

## Independent LLM Capability

shittim-chest has its own LLM routing layer that operates independently of entelecheia:

- **LlmRouter**: Multi-provider router with priority-based selection and fallback
- **Provider management**: CRUD endpoints for adding/editing/removing LLM providers
- **API key encryption**: Provider API keys are encrypted at rest with AES-256-GCM
- **OpenAI-compatible**: Works with any OpenAI-compatible API (DeepSeek, OpenAI, local models, etc.)
- **Dual streaming**: SSE (Server-Sent Events) and WebSocket streaming for chat responses

This means shittim-chest can run as a standalone chat application without entelecheia, or use entelecheia agents via the proxy layer.

## Auth Flow

### Login Sequence

```text
User → shittim_chest: POST /api/auth/login { username, password }
shittim_chest → shittim_chest_db: SELECT user WHERE username = ? (verify argon2 hash)
shittim_chest → scepter: GET /api/user/{id}/permissions
scepter → entelecheia_db: query groups + permissions
scepter → shittim_chest: { groups: [...], permissions: {...} }
shittim_chest → User: { access_token, refresh_token }
shittim_chest: Store session + cache RBAC
```

### GitHub OAuth

```text
User → shittim_chest: GET /api/auth/github
shittim_chest → User: 302 redirect to GitHub OAuth
User → GitHub: authorize
GitHub → shittim_chest: GET /api/auth/github/callback?code=...
shittim_chest → GitHub: exchange code for access token
shittim_chest → GitHub: GET /user (fetch user info)
shittim_chest → shittim_chest_db: INSERT/UPDATE oauth_connections
shittim_chest → User: { access_token, refresh_token } (auto-creates user if new)
```

## Chat Architecture

### Message Flow (Standalone LLM)

```text
User → POST /api/chat/conversations/:id/messages
shittim_chest: validate JWT, load conversation
shittim_chest → LlmRouter: route request to best provider
LlmRouter → LLM Provider: POST chat/completions (streaming)
LLM Provider → LlmRouter: SSE stream
LlmRouter → User: SSE/WS stream (tokens as they arrive)
shittim_chest: persist message to shittim_chest_db
```

### SSE vs WebSocket Streaming

- **SSE** (`/api/chat/stream`): Simple HTTP streaming, works through proxies, auto-reconnect
- **WebSocket** (`/ws/chat/stream`): Bidirectional, supports cancellation and real-time interaction

## Proxy Architecture

The `/api/proxy/*` endpoint forwards authenticated requests to scepter:

1. Browser opens `ws://shittim-chest:80/api/proxy/chat` with JWT
1. `shittim_chest` validates JWT, opens connection to scepter forwarding the JWT
1. Bidirectional message forwarding between browser and scepter
1. `shittim_chest` enforces rate limits, logs usage, manages connection lifecycle

## Webhook Pipeline

Webhooks from external services enter through `/api/webhook/*`:

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC validation → Parse event → Forward to scepter via Unix socket
```

Supported sources: GitHub (HMAC-SHA256), GitLab (token), Gitee (HMAC + token fallback), plus a generic `/api/webhook/custom/{name}` endpoint. Features:

- Duplicate delivery detection (LRU cache, 10,000 IDs)
- Delivery log with listing API
- IP whitelist for webhook sources

## Remote Device Management

Remote devices are managed through a signaling relay:

```text
Browser (webui) → WS /api/devices/stream → shittim_chest (signal relay) → Unix socket → entelecheia/polemos
```

Features:

- Device listing and session CRUD via REST
- WebRTC signaling (SDP offer/answer, ICE candidates)
- Terminal relay (WebSocket to xterm.js)
- Desktop frame relay
- SFTP file browser backend

shittim-chest never connects to remote devices directly — all data flows through entelecheia's polemos agent.

## Data Ownership

### shittim_chest_db

| Data | Table | Rationale |
| --- | --- | --- |
| Password hashes (argon2) | `auth_users` | Presentation layer owns login flow |
| Active sessions, refresh tokens | `sessions` | Session management is a frontend concern |
| Encrypted API keys | `api_keys` | API key issuance is user-facing |
| OAuth connections | `oauth_connections` | Third-party auth binding is user-facing |
| Conversations, messages | `conversations`, `messages` | Chat data is user-facing |
| LLM provider configs | `llm_providers` | Provider management is user-facing (keys encrypted) |
| Remote device records | `remote_devices`, `device_sessions` | Device tracking is user-facing |
| Channel configs | `channel_configs`, etc. | Multi-platform config is user-facing |

### entelecheia_db

| Data | Rationale |
| --- | --- |
| User identity, groups, role assignments | Core enforces permissions |
| GroupPermissions (provider quotas, agent whitelists) | Agent-level policy lives with agents |
| Agent configurations, Cosmos/IEPL state | Orchestration data belongs to the core |

## Dual Frontend Strategy

### Phase 1: Vue 3 (Current)

| Package | Tech | Port | Purpose |
| --- | --- | --- | --- |
| `webui` | Vue 3 + Vite + Pinia (TSX) | `:3000 (shared)` | Unified webui: chat, image gen, devices, admin (providers, agents, RBAC, webhooks) |

### Phase 2: Rust WASM (Future)

| Package | Tech | Purpose |
| --- | --- | --- |
| `webui` | Rust → WASM (Tairitsu) | Long-term unified webui (chat + admin) |

The legacy frontends serve as living specifications. During transition, both versions run in parallel, and identical user interactions must produce identical outcomes.

## Reverse Proxy Deployment Modes

shittim-chest supports three reverse proxy modes, controlled by `SHITTIM_CHEST_PROXY_MODE` in `.env`.

### Mode 1: None (Direct)

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=none   # or unset
```

The core server binds directly to `SHITTIM_CHEST_HOST:SHITTIM_CHEST_PORT` (default `0.0.0.0:80`). No TLS, no reverse proxy container. Suitable for:

- Local development
- Behind an existing reverse proxy (Cloudflare Tunnel, AWS ALB, Traefik labels)
- Docker networks where another service handles TLS termination

### Mode 2: Caddy Auto

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_DOMAIN=app.example.com
```

The CLI creates a `shittim-chest-caddy` container (image `caddy:2`) that:

1. Listens on ports 80/443 (configurable via `SHITTIM_CHEST_PROXY_HTTP_PORT` / `SHITTIM_CHEST_PROXY_HTTPS_PORT`)
1. Automatically provisions TLS certificates via Let's Encrypt (Caddy's built-in ACME)
1. Proxies all requests to the core backend on the Docker network

No Caddyfile needed — the CLI generates one automatically. The domain must have public DNS pointing to the host.

### Mode 3: Caddy Custom

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/caddy/Caddyfile
SHITTIM_CHEST_PROXY_EXTRA_VOLUMES=/etc/letsencrypt:/etc/letsencrypt
```

Same Caddy container, but you provide your own Caddyfile (mounted from the host). Use this when you need:

- Multiple virtual hosts
- Custom TLS certificate paths
- Additional middleware (basic auth, rate limiting, etc.)
- Serving static files alongside the API

### Mode 4: Nginx Custom

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=nginx
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/nginx/conf.d/default.conf
```

Creates an `nginx:bookworm` container with your config file. You manage TLS certificates yourself. Suitable for environments where Nginx is the standard.

### Container Lifecycle

All proxy containers are managed by the CLI via the Docker API (`bollard`):

| Command | Behavior |
| --- | --- |
| `just dev` / `chest up` | Creates/starts proxy container if `PROXY_MODE` is set |
| `just dev-stop` / `chest down` | Stops and removes proxy container |
| Container already running | Reuses existing container (idempotent) |

The proxy container joins the same Docker network as the core backend, so it reaches the backend via the internal hostname (`core` or `shittim-chest`).
