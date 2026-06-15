<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Arona logo" width="200"/>

# Arona

**Shared JSON-RPC 2.0 Protocol Types for the Entelecheia Multi-Agent Platform**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](README.md)** &bull; **[简体中文](docs/guides/zhs/README.md)** &bull;
**[繁體中文](docs/guides/zht/README.md)** &bull; **[日本語](docs/guides/ja/README.md)** &bull;
**[한국어](docs/guides/ko/README.md)** &bull; **[Français](docs/guides/fr/README.md)** &bull;
**[Español](docs/guides/es/README.md)** &bull; **[Русский](docs/guides/ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — Consumed by [entelecheia](https://github.com/celestia-island/entelecheia) and [shittim-chest](https://github.com/celestia-island/shittim-chest).

JSON-RPC 2.0 message types, agent taxonomy (14 variants), ~100 WebSocket parameter types. Auto-generated TypeScript bindings via `ts-rs`.

## Usage

**Rust:**
```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript:**
```bash
pnpm add @celestia-island/arona
```

## License

Business Source License 1.1 — non-commercial use under Apache-2.0 or MIT.
