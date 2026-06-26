# Arona Documentation

Unified documentation hub for the **celestia-island** ecosystem.

## Structure

| Directory | Contents |
|-----------|----------|
| [`meta/`](meta/) | Governance & legal: CLA, Code of Conduct, Security policy, Contributing guide |
| [`architecture/`](architecture/) | High-level architecture overviews |
| [`architecture/core/`](architecture/core/) | **entelecheia** — agent orchestration backend |
| [`architecture/webui/`](architecture/webui/) | **shittim-chest** — user-facing shell & frontend |
| [`design/`](design/) | Design documents & technical RFCs |
| [`design/core/`](design/core/) | **entelecheia** — 29 design docs (agent system, IEPL, containers, etc.) |
| [`design/webui/`](design/webui/) | **shittim-chest** — design docs (deployment, LLM architecture, RBAC, etc.) |
| [`guides/`](guides/) | User & contributor guides |
| [`guides/core/`](guides/core/) | **entelecheia** — building, CLI, agent development, MCP tools, etc. |
| [`guides/webui/`](guides/webui/) | **shittim-chest** — architecture, building, fundamentals, webhooks |
| [`licenses/`](licenses/) | Translated legal documents (11 languages) |

## Repositories

- **[arona](https://github.com/celestia-island/arona)** — this repo. Shared protocol types, TypeScript bindings, devtools, and documentation hub.
- **[entelecheia](https://github.com/celestia-island/entelecheia)** — Rust-based multi-agent collaboration platform (the "core").
- **[shittim-chest](https://github.com/celestia-island/shittim-chest)** — User-facing shell: web UI, backend, CLI, IDE plugins (the "webui").

## Conventions

- `core/` subdirectories contain documentation for **entelecheia**.
- `webui/` subdirectories contain documentation for **shittim-chest**.
- Only English source documents live here; per-repo translated copies remain in their respective repositories.
