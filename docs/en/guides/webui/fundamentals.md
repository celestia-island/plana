+++
title = "Core Concepts"
description = """> Audience: Developers who want a conceptual understanding of shittim-chest's design."""
lang = "en"
category = "guides"
subcategory = "webui"
+++

# Core Concepts

> **Audience**: Developers who want a conceptual understanding of shittim-chest's design.
> **Last updated**: 2026-05-25

## Two-Repo Architecture

shittim-chest and [entelecheia](https://github.com/celestia-island/entelecheia) form a two-repo system with a deliberate boundary:

- **entelecheia** — agent orchestration core (scepter, 13 agents, Cosmos/IEPL runtime). Owns identity, permissions, agent configurations.
- **shittim-chest** — user-facing shell. Owns authentication, sessions, chat data, LLM provider config, frontend UI, and the proxy bridge to scepter.

They communicate via JWT-authenticated HTTP and WebSocket. Neither accesses the other's database directly. This separation allows each repo to be developed, deployed, and scaled independently.

## Dual Operation Modes

shittim-chest supports two operation modes:

### Standalone Mode

Runs independently with its own LLM routing layer. Supports:

- Chat with streaming responses (SSE + WebSocket)
- Image generation via configured providers
- User authentication (password + GitHub OAuth)
- Provider management (add/remove LLM providers)

Does not require entelecheia. Useful for development and simple deployments.

### Proxy Mode

Acts as a gateway to entelecheia's agent system. Adds:

- Request forwarding to scepter with JWT passthrough
- WebSocket bridging for agent-based chat
- Webhook ingress and trigger forwarding
- Remote device management via polemos
- RBAC permission queries and caching

Requires a running entelecheia instance. The two modes can coexist — standalone LLM for simple chat, proxy for agent orchestration.

## Authentication Model

Authentication uses JWT tokens issued by `shittim_chest`:

1. **Credential storage**: Passwords (argon2 hashes), sessions, refresh tokens, and API keys live in `shittim_chest_db`.
1. **GitHub OAuth**: Users can sign in with GitHub; accounts are auto-created on first login.
1. **Permission storage**: User groups, roles, and permission matrices live in `entelecheia_db`.
1. **JWT flow**: On login, `shittim_chest` verifies credentials locally, then fetches permissions from scepter. The issued JWT contains `{ sub: user_id, groups: [...] }`.
1. **Shared secret**: The JWT signing secret is shared with scepter so both services can validate tokens independently.
1. **Token rotation**: Access tokens expire in 1 hour; refresh tokens in 7 days. Refresh tokens are rotated on each use.

## Frontend (webui)

The webui is the unified frontend at `packages/webui/`, with the chat interface at `/` and the admin panel at `/backend`, built with Vue 3 + Vite + Pinia (TSX via `@vitejs/plugin-vue-jsx`).

## LLM Provider System

shittim-chest has an independent LLM routing layer:

- **Providers**: Configurable LLM API endpoints (OpenAI-compatible). Stored in `shittim_chest_db` with AES-256-GCM encrypted API keys.
- **Router**: Multi-provider routing with priority-based selection and automatic fallback.
- **Categories**: Providers can be tagged as `chat`, `image`, or both.
- **Management**: Full CRUD via REST API and webui admin panel. Providers can be tested for connectivity.
- **Streaming**: Both SSE (simple, proxy-friendly) and WebSocket (bidirectional) streaming protocols.

## Chat System

- **Conversations**: Thread-based chat sessions with titles and metadata
- **Messages**: Supports text, images, and tool calls (function calling)
- **Streaming**: Real-time token-by-token response delivery via SSE or WebSocket
- **Search**: Full-text message search with ILIKE queries
- **Export**: Conversations can be exported as JSON or Markdown format
- **Image Generation**: Prompt-based image generation via configured providers, with "Insert to chat" functionality

## Remote Device Management

shittim-chest provides a browser-based interface for remote devices managed by entelecheia/polemos:

- **Desktop**: WebRTC-based remote desktop viewer with frame relay
- **Terminal**: xterm.js-based terminal emulator with WebSocket relay
- **File Browser**: SFTP file browser backend (skeleton)
- **Signaling**: WebSocket-based WebRTC signaling relay (SDP offer/answer, ICE candidates)

All device communication flows through entelecheia's polemos agent — shittim-chest never connects directly to endpoints.

## Proxy Architecture

`shittim_chest` acts as a gateway between users and scepter:

- **HTTP reverse proxy**: `/api/proxy/*` forwards authenticated requests to scepter with JWT passthrough.
- **WebSocket bridge**: Chat streaming uses bidirectional WebSocket forwarding (`browser ↔ shittim_chest ↔ scepter`).

This allows `shittim_chest` to enforce rate limits, log usage, and manage connection lifecycle without scepter needing to handle individual browser connections.

## Webhook Pipeline

External events reach the agent core through a webhook pipeline:

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC validation → Parse event → Forward to scepter via Unix socket → Agent dispatch
```

Each provider has its own validation mechanism:

- **GitHub**: HMAC-SHA256 via `X-Hub-Signature-256`
- **GitLab**: Token via `X-Gitlab-Token`
- **Gitee**: HMAC with token fallback

Additional features: duplicate delivery detection (LRU cache), delivery logging, IP whitelist, and a generic custom webhook endpoint.

## RBAC Model

Permissions follow a group-based RBAC model:

- **Groups**: Users belong to one or more groups.
- **Roles**: Groups have assigned roles.
- **Permissions**: Each role defines a permission matrix covering:
  - Provider quotas (max tokens, max requests)
  - Agent whitelists (which agents the group can access)
  - Administrative capabilities (manage users, configure providers)

`shittim_chest` caches permissions in-process with a TTL (default 5 minutes). Cache invalidation occurs on TTL expiry, logout, or explicit permission changes propagated from scepter.

## Frontend Strategy

shittim-chest uses a two-phase frontend approach:

**Phase 1 (current)**: Vue 3 frontend (`webui`, at `packages/webui/`) built with Vite + Pinia, using TSX via `@vitejs/plugin-vue-jsx`. It defines the API contract and serves as a production-quality reference implementation.

**Phase 2 (future)**: Rust → WASM frontend built with Tairitsu. The legacy frontend acts as a living specification and test oracle — identical user interactions must produce identical results.

## Type Safety Bridge

TypeScript types are generated from Rust code via the external `arona` protocol crate, ensuring frontend-backend consistency:

```text
arona Rust crate (git dependency)
  → #[derive(ts_rs::TS)]
  → ts-rs codegen → packages/webui/src/types/arona/ (TypeScript)
  → consumed by webui as @celestia-island/arona
```

This eliminates manual type synchronization. When a Rust type in the `arona` crate changes, the TypeScript bindings are regenerated and consumed by the webui.
