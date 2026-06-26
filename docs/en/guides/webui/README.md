+++
title = "Shittim Chest (什亭之匣)"
description = """User-facing shell for the [entelecheia](https://github.com/celestia-island/entelecheia) multi-agent platform"""
lang = "en"
category = "guides"
subcategory = "webui"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Shittim Chest logo" width="200"/>

# Shittim Chest (什亭之匣)

**User-facing shell for the [entelecheia](https://github.com/celestia-island/entelecheia) multi-agent platform**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fshittim--chest-blue.svg)](https://github.com/celestia-island/shittim-chest)

**[English](README.md)** &bull; **[简体中文](docs/guides/zhs/README.md)** &bull;
**[繁體中文](docs/guides/zht/README.md)** &bull; **[日本語](docs/guides/ja/README.md)** &bull;
**[한국어](docs/guides/ko/README.md)** &bull; **[Français](docs/guides/fr/README.md)** &bull;
**[Español](docs/guides/es/README.md)** &bull; **[Русский](docs/guides/ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — Active development.

Webui, backend, and CLI for the [Entelecheia](https://github.com/celestia-island/entelecheia) multi-agent platform. Includes chat, admin panel, auth, multi-channel integrations, and device management.

## Quick Start

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
just dev    # backend on :3000, frontend on :5173
```

**Prerequisites**: Rust 1.85+, Node 20+, pnpm 9+, [just](https://github.com/casey/just), PostgreSQL 18+.

**[Architecture](ARCHITECTURE.md)** · **[Contributing](CONTRIBUTING.md)** · **[Security](SECURITY.md)** · **[Docs](docs/guides/en/)**

## License

Business Source License 1.1 — commercial use requires a license. Non-commercial use under the Synthetic Source License (SySL-1.0); converts fully to SySL-1.0 on 2030-01-01.
