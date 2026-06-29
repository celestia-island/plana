<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Arona logo" width="200"/>

# Arona

**Shared protocol types, TypeScript bindings, devtools, and documentation hub for the Entelecheia Multi-Agent Platform**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — Consumed by [entelecheia](https://github.com/celestia-island/entelecheia) and [shittim-chest](https://github.com/celestia-island/shittim-chest).

## Repository Structure

Arona is a single Rust crate that also ships generated TypeScript bindings,
shared devtools, and the unified documentation hub for the celestia-island
ecosystem:

```text
arona/
├── src/                 # Rust crate — JSON-RPC 2.0 & API types (the `arona` package)
├── examples/            # Runnable examples (e.g. schema dump)
├── bindings/            # TypeScript bindings (auto-generated via ts-rs)
├── devtools/            # Shared Python build scripts
├── docs/                # Unified documentation for the entire ecosystem
│   ├── meta/            # CLA, CoC, SECURITY, CONTRIBUTING (canonical)
│   ├── architecture/    # Architecture overviews (core/, webui/)
│   ├── design/          # Design documents (core/, webui/)
│   ├── guides/          # User & contributor guides (core/, webui/)
│   └── licenses/        # Translated legal documents
└── ...
```

> The Python devtools and TypeScript bindings live in the repo but are kept
> out of the published Rust crate via the `exclude` list in `Cargo.toml`.

## Components

### `src/` — Rust crate (`arona`)

JSON-RPC 2.0 message types, agent taxonomy (16 variants), ~230 WebSocket/HTTP parameter types.

**Rust usage:**

```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

### `bindings/` (TypeScript)

Auto-generated TypeScript bindings from the Rust crate via `ts-rs`.

```bash
pnpm add @celestia-island/arona
```

### `devtools/` (Python)

Shared build/dev scripts (cargo cache guard, logger, offline prefetch) used across all celestia-island repos.

## License

Business Source License 1.1 — non-commercial use under Apache-2.0 or MIT.
