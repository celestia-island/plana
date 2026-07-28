<!-- markdownlint-disable MD033 MD041 MD036 -->
<p align="center"><img src="https://raw.githubusercontent.com/celestia-island/docs.celestia.world/master/res/logo/plana.webp" alt="Plana" width="200" /></p>

<h1 align="center">Plana</h1>

<p align="center"><strong>JSON-RPC 2.0 protocol types &amp; TypeScript bindings</strong></p>

<div align="center">

[![License: SySL-1.0](https://img.shields.io/badge/License-SySL--1.0-blue.svg)](https://sysl.celestia.world)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fplana-blue.svg)](https://github.com/celestia-island/plana)

</div>

<div align="center">

**English** ·
[简体中文](https://github.com/celestia-island/docs.celestia.world/blob/master/docs/zh-Hans/guides/platforms/README-plana.md) ·
[繁體中文](https://github.com/celestia-island/docs.celestia.world/blob/master/docs/zh-Hant/guides/platforms/README-plana.md) ·
[日本語](https://github.com/celestia-island/docs.celestia.world/blob/master/docs/ja/guides/platforms/README-plana.md) ·
[한국어](https://github.com/celestia-island/docs.celestia.world/blob/master/docs/ko/guides/platforms/README-plana.md) ·
[Français](https://github.com/celestia-island/docs.celestia.world/blob/master/docs/fr/guides/platforms/README-plana.md) ·
[Español](https://github.com/celestia-island/docs.celestia.world/blob/master/docs/es/guides/platforms/README-plana.md) ·
[Русский](https://github.com/celestia-island/docs.celestia.world/blob/master/docs/ru/guides/platforms/README-plana.md)

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

JSON-RPC 2.0 protocol types, TypeScript bindings, and the documentation hub. Consumed by entelecheia and shittim-chest.

## Quick Start

```bash
# Build
cargo build

# Run all tests (includes TS binding generation)
cargo test --all-features

# Check lint + formatting
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Generate TypeScript bindings only
cargo test --package plana
```

Or use the [just](https://github.com/casey/just) task runner:

```bash
just build
just test
just fmt-check
```

## Documentation

Architecture, design, and guides live at [docs.celestia.world](https://github.com/celestia-island/docs.celestia.world/tree/master/docs/en/guides/platforms).

Source: [plana](https://github.com/celestia-island/plana).
