+++
title = "Embedded Frontend Strategy"
description = """shittim-chest supports two frontend hosting modes: in Dev mode, `dev.py` watches frontend sources and triggers `pnpm build` on changes, with the backend serving both static files and API on `:3000`; i"""
lang = "en"
category = "design"
subcategory = "webui"
+++

# Embedded Frontend Strategy

## Overview

shittim-chest supports two frontend hosting modes: in Dev mode, `dev.py` watches frontend sources and triggers `pnpm build` on changes, with the backend serving both static files and API on `:3000`; in Release mode, frontend static files are embedded into the Rust binary at compile time and served on `:80`. The modes are switched via the `embedded-frontend` Cargo feature, with code-level conditional compilation using `#[cfg(feature = "embedded-frontend")]`.

## Architecture Comparison

```mermaid
flowchart TB
    subgraph Dev[Dev Mode: dev.py + Backend]
        D1[dev.py watches frontend src] --> D2[pnpm build → dist/]
        D2 --> D3[shittim_chest :3000 serves static + API]
    end
    subgraph Release[Release Mode: Embedded]
        R1[Browser] --> R2[shittim_chest :80]
        R2 --> R3[API + LLM]
        R2 --> R4[/static/*\nEmbedded SPA]
    end
```

| Dimension | Dev (no feature) | Release (embedded-frontend) |
| --- | --- | --- |
| Frontend source | Built by Vite, served by backend | `include_dir!` compile-time embedding |
| Hot reload | Auto-rebuild via dev.py | Not supported (static) |
| API request routing | Browser direct connection (same origin) | Browser direct connection |
| Binary size | Backend only | + frontend dist/ directory |
| Requires Node | Yes (build only) | No |
| Startup method | `dev.py` (watches + rebuilds) | `just up` one-shot launch |

## Implementation Details

### Conditional Compilation

```rust
# [cfg(feature = "embedded-frontend")]
static ARONA_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../dist/arona");

async fn serve_arona() -> impl IntoResponse {
    #[cfg(feature = "embedded-frontend")]
    {
        // Read from compile-time embedded Dir
    }
    #[cfg(not(feature = "embedded-frontend"))]
    {
        // Read from filesystem ./dist/arona/index.html
    }
}
```

Conditional compilation operates at the **function body level** rather than the module level, keeping the public API identical across both modes.

### SPA Fallback

The application is a single-page application. All routes not matching static assets return `index.html`:

```text
GET /               → index.html
GET /chat/123       → index.html (frontend router handles)
GET /backend        → index.html
GET /backend/providers → index.html (frontend router handles)
```

### MIME Type Detection

Static file serving returns the correct Content-Type based on file extension:

| Extension | Content-Type |
| --- | --- |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.html` | `text/html` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff/.woff2` | `font/woff2` |
| Other | `application/octet-stream` |

## Frontend Build in Dockerfile

```text
Stage 1 (frontend):
  node:22-slim → pnpm install → pnpm build:all → /app/dist/arona/

Stage 2 (builder):
  rust:1.85-slim → COPY /app/dist/ → cargo build --features embedded-frontend

Stage 3 (runtime):
  debian:bookworm-slim → COPY binary → ENTRYPOINT ["./shittim_chest"]
```

Frontend build and Rust compilation are completed within the same Dockerfile. The final runtime image contains only the compiled binary.

## Design Decisions

1. **Dev mode uses dev.py for auto-rebuild**: `dev.py` watches frontend sources and rebuilds on changes, with the backend serving everything on one port.
1. **Release mode does not require a reverse proxy**: The binary embeds the SPA, enabling single-process deployment and reducing operational complexity.
1. **Frontend is not dynamically loaded at runtime**: Avoids filesystem dependencies and version inconsistency. The Release image contains only a single binary file.
1. **Single SPA**: The frontend is served at `/` with the admin panel at `/backend`.
