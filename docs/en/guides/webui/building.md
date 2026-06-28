+++
title = "Building and Development Guide"
description = """> Audience: Contributors setting up a local shittim-chest development environment."""
lang = "en"
category = "guides"
subcategory = "webui"
+++

# Building and Development Guide

> **Audience**: Contributors setting up a local shittim-chest development environment.
> **Last updated**: 2026-05-25

## Prerequisites

| Tool | Minimum Version | Notes |
| --- | --- | --- |
| Rust | 1.85+ | Edition 2024 required; install via <https://rustup.rs> |
| Node.js | 20+ | LTS recommended |
| pnpm | 9+ | `corepack enable && corepack prepare pnpm@latest --activate` |
| just | latest | Command runner; `cargo install just` |
| PostgreSQL | 18+ | shittim_chest_db for auth + chat data |
| entelecheia scepter | optional | Required for proxy/device features; optional for standalone chat |

Verify everything:

```bash
rustc --version    # >= 1.85
node --version     # >= 20
pnpm --version     # >= 9
just --version
psql --version     # >= 18
```

## Clone and Bootstrap

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
```

## Environment Variables

Edit `.env` after cloning. Every variable is documented inline; below is a summary.

### Server

| Variable | Default | Purpose |
| --- | --- | --- |
| `SHITTIM_CHEST_HOST` | `0.0.0.0` | Listen address |
| `SHITTIM_CHEST_PORT` | `80` | Listen port |

### Database

| Variable | Default | Purpose |
| --- | --- | --- |
| `SHITTIM_CHEST_DATABASE_URL` | `postgresql://sc:pass@localhost:5432/shittim_chest` | PostgreSQL connection string |
| `SHITTIM_CHEST_DATABASE_MAX_CONNECTIONS` | `10` | SeaORM connection pool size |

Create the database and user:

```sql
CREATE USER sc WITH PASSWORD 'pass';
CREATE DATABASE shittim_chest OWNER sc;
```

### JWT & Encryption

| Variable | Default | Purpose |
| --- | --- | --- |
| `JWT_SECRET` | `change-me-in-production` | Shared secret with scepter; **must match** |
| `JWT_EXPIRATION_SECONDS` | `3600` | Access token lifetime (1 hour) |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | Refresh token lifetime (7 days) |
| `SHITTIM_CHEST_ENCRYPTION_KEY` | `change-me-32-bytes-base64-encoded` | AES-256-GCM key for API keys and OAuth tokens |

Generate a production key:

```bash
openssl rand -base64 32
```

### LLM Providers (for standalone operation)

Set these to use shittim-chest independently without entelecheia:

| Variable | Purpose |
| --- | --- |
| `LLM_DEFAULT_PROVIDER_ENDPOINT` | OpenAI-compatible API endpoint (e.g. `https://api.deepseek.com/v1`) |
| `LLM_DEFAULT_PROVIDER_API_KEY` | API key for the provider |
| `LLM_DEFAULT_PROVIDER_MODELS` | Comma-separated model list (e.g. `deepseek-chat,deepseek-reasoner`) |
| `LLM_DEFAULT_PROVIDER_CATEGORY` | Provider category: `chat` or `image` |
| `LLM_STREAM_BUFFER_SECONDS` | Stream buffer timeout (default: 60) |
| `LLM_MAX_TOKENS_DEFAULT` | Default max tokens (default: 4096) |
| `LLM_REQUEST_TIMEOUT_SECONDS` | HTTP request timeout (default: 120) |

### Remote Devices

| Variable | Default | Purpose |
| --- | --- | --- |
| `REMOTE_DEVICES_ENABLED` | `false` | Enable remote device features |
| `REMOTE_DEVICES_SCEPTER_SOCK` | `/run/entelecheia/device_stream.sock` | Unix socket for device data |
| `REMOTE_DEVICES_FRAME_BUFFER_SIZE` | `4194304` | Frame buffer size in bytes |
| `REMOTE_DEVICES_MAX_SESSIONS_PER_USER` | `3` | Max concurrent device sessions |
| `WEBRTC_ICE_SERVERS` | `stun:stun.l.google.com:19302` | ICE server list |

### GitHub OAuth

| Variable | Purpose |
| --- | --- |
| `GITHUB_CLIENT_ID` | GitHub OAuth App client ID |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth App client secret |
| `GITHUB_REDIRECT_URI` | OAuth callback URL (e.g. `https://your-domain/api/auth/github/callback`) |

### Scepter Connection (for proxy features)

| Variable | Default | Purpose |
| --- | --- | --- |
| `ENTELECHEIA_SCEPTER_URL` | `http://localhost:8424` | HTTP endpoint for scepter |
| `ENTELECHEIA_SCEPTER_WS_URL` | `ws://localhost:8424` | WebSocket endpoint for scepter |
| `ENTELECHEIA_TUI_SOCK` | `/run/entelecheia/entelecheia.sock` | Unix socket for trigger forwarding |

### Webhook

| Variable | Purpose |
| --- | --- |
| `WEBHOOK_GITHUB_SECRET` | HMAC secret for GitHub webhook validation |
| `WEBHOOK_GITLAB_SECRET` | Token for GitLab webhook validation |
| `WEBHOOK_PUBLIC_URL` | Public-facing URL for webhook endpoints |

## Database Setup

```bash
just db-init      # Create schema (runs SeaORM migrations)
just db-migrate   # Apply pending migrations
```

### Schema Overview

The `shittim_chest_db` owns user-facing data:

| Table | Purpose |
| --- | --- |
| `auth_users` | User accounts with argon2 password hashes |
| `sessions` | Active sessions with refresh tokens |
| `api_keys` | API key records (hashed) |
| `oauth_connections` | Third-party OAuth bindings (GitHub) |
| `conversations` | Chat conversations |
| `messages` | Chat messages with tool call data |
| `llm_providers` | LLM provider configurations (API keys encrypted) |
| `remote_devices` | Remote device records |
| `device_sessions` | Active device sessions |
| `channel_configs` | Multi-platform channel configs |
| `channel_messages` | Channel message records |
| `channel_pairings` | Channel-to-chat pairings |

Reset the database:

```bash
just db-reset
```

## Backend Development

```bash
just dev-backend
```

This runs `cargo run --package shittim_chest`. The server starts on `:80`.

### CLI Commands

```bash
shittim_chest db-init      # Create database schema
shittim_chest db-migrate   # Apply pending migrations
shittim_chest db-reset     # Drop and recreate schema
shittim_chest server       # Start the web server
```

### Hot Reload

```bash
cargo install cargo-watch
cargo watch -x 'run --package shittim_chest -- server'
```

### API Endpoints Overview

| Route Group | Purpose |
| --- | --- |
| `/api/auth/*` | Login, register, GitHub OAuth, refresh, logout |
| `/api/chat/*` | Conversations, messages, SSE/WS streaming, search, export |
| `/api/providers/*` | LLM provider CRUD, API key management, testing |
| `/api/generation/*` | Image generation, model listing |
| `/api/devices/*` | Remote device listing, sessions, WebRTC signaling |
| `/api/webhook/*` | GitHub/GitLab/Gitee/custom webhook ingress |
| `/api/proxy/*` | Reverse proxy to scepter (HTTP + WebSocket) |
| `/static/*` | SPA static file hosting |

## Frontend Development

### Install Dependencies

```bash
pnpm install
```

### webui

```bash
just dev    # build frontend + start backend on :3000
just watch  # auto-rebuild on file changes
```

Both frontends are built by Vite into `dist/`. The backend serves these static files directly on `:3000` — no separate Vite dev server or proxy is needed. In dev mode, `dev.py` watches frontend sources and rebuilds automatically.

## Cross-Project Setup

For local development with the shared `arona` protocol crate, patch it to your local checkout. Edit `~/.cargo/config.toml` (never committed to the repo):

```toml
[patch.'https://github.com/celestia-island/arona']
arona = { path = "/path/to/arona" }
```

For npm, the webui consumes the `arona` crate's TS bindings via the `@celestia-island/arona` path alias, pointing at `packages/webui/src/types/arona/`.

## Building for Production

```bash
just build
```

This runs `cargo build --release` and `pnpm run build:all`. Output locations:

- Backend binary: `target/release/shittim_chest`
- Frontend assets: `packages/webui/dist/`

### Docker

Build and run with the CLI wrapper (uses Docker API directly):

```bash
just dev
```

Or manually:

```bash
just build        # build Docker image
just up           # start all services
just migrate      # run database migrations
```

The production binary serves frontend assets via Axum's static file middleware at `/`. No separate frontend server is needed.

## Common Issues

### Database connection refused

```text
error: connection to server at "localhost", port 5432 failed
```

**Fix**: Ensure PostgreSQL is running and `SHITTIM_CHEST_DATABASE_URL` in `.env` matches your setup. Verify with `psql "$SHITTIM_CHEST_DATABASE_URL" -c 'SELECT 1'`.

### Scepter not reachable

```text
error: error sending request for url (http://localhost:8424/...)
```

**Fix**: Start the entelecheia scepter instance, or use standalone mode with LLM providers configured. The backend works without scepter for chat/image generation.

### CORS errors in browser

```text
Access-Control-Allow-Origin header is present on the requested resource
```

**Fix**: The dev backend enables CORS for `localhost` origins. If you changed ports, update the CORS config. Production deployments should configure a reverse proxy (nginx/caddy) to handle CORS.

### pnpm install fails

**Fix**: Ensure you are using pnpm 9+. Run `corepack enable && corepack prepare pnpm@latest --activate` to set up the correct version.

### cargo build fails on shared crates

**Fix**: If you have local patches in `~/.cargo/config.toml`, ensure the paths exist and the crate names match. Remove the patch section to use git dependencies instead.
