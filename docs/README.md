# Arona Documentation

Unified documentation hub for the **celestia-island** ecosystem.

## Structure

```text
docs/
├── PLAN.md                # i18n & format alignment roadmap
├── README.md              # This index
├── logo.webp              # Arona logo
├── licenses/              # Translated legal documents (11 languages)
├── en/                    # English (canonical source)
│   ├── meta/              # Governance & legal
│   ├── architecture/      # High-level architecture
│   │   ├── core/          # entelecheia — agent orchestration backend
│   │   ├── webui/         # shittim-chest — user-facing shell
│   │   └── router/        # evernight — remote control & protocol broker
│   ├── design/            # Design documents & technical RFCs
│   │   ├── core/          # entelecheia
│   │   ├── webui/         # shittim-chest
│   │   └── router/        # evernight
│   └── guides/            # User & contributor guides
│       ├── core/          # entelecheia
│       ├── webui/         # shittim-chest
│       └── router/        # evernight
├── zh/                    # Simplified Chinese
├── zht/                   # Traditional Chinese
├── ja/                    # Japanese
├── ko/                    # Korean
├── fr/                    # French
├── es/                    # Spanish
└── ru/                    # Russian
```

## Repositories

- **[arona](https://github.com/celestia-island/arona)** — this repo. Shared protocol types, TypeScript bindings, devtools, and documentation hub.
- **[entelecheia](https://github.com/celestia-island/entelecheia)** — Rust-based multi-agent collaboration platform (the "core").
- **[shittim-chest](https://github.com/celestia-island/shittim-chest)** — User-facing shell: web UI, backend, CLI, IDE plugins (the "webui").
- **[evernight](https://github.com/celestia-island/evernight)** — Cross-platform remote control library & daemon: screen streaming, SSH/VNC/RDP, hardware telemetry, industrial protocols (Modbus/S7comm/OPC UA), and S7 self-networking (the "router").

## Conventions

- `en/` is the canonical source; translations live in their respective language directories.
- `core/` subdirectories contain documentation for **entelecheia**.
- `webui/` subdirectories contain documentation for **shittim-chest**.
- `router/` subdirectories contain documentation for **evernight** — the remote-control and industrial-protocol broker. The TIA Portal setup guide (`guides/router/tia-portal-setup.md`) covers the one-time PLC preparation for S7 self-networking.
- Each document begins with TOML frontmatter (`+++` / `+++`).
