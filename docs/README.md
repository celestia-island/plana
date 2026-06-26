# Arona Documentation

Unified documentation hub for the **celestia-island** ecosystem.

## Structure

```
docs/
├── PLAN.md                # i18n & format alignment roadmap
├── README.md              # This index
├── logo.webp              # Arona logo
├── licenses/              # Translated legal documents (11 languages)
├── en/                    # English (canonical source)
│   ├── meta/              # Governance & legal
│   ├── architecture/      # High-level architecture
│   │   ├── core/          # entelecheia — agent orchestration backend
│   │   └── webui/         # shittim-chest — user-facing shell
│   ├── design/            # Design documents & technical RFCs
│   │   ├── core/          # entelecheia
│   │   └── webui/         # shittim-chest
│   └── guides/            # User & contributor guides
│       ├── core/          # entelecheia
│       └── webui/         # shittim-chest
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

## Conventions

- `en/` is the canonical source; translations live in their respective language directories.
- `core/` subdirectories contain documentation for **entelecheia**.
- `webui/` subdirectories contain documentation for **shittim-chest**.
- Each document begins with TOML frontmatter (`+++` / `+++`).
