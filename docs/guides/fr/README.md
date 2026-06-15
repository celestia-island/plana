<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Arona logo" width="200"/>

# Arona

**Types de protocole JSON-RPC 2.0 partagés pour la plateforme multi-agent Entelecheia**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[Anglais](../../README.md)** &bull; **[Chinois simplifié](../zhs/README.md)** &bull;
**[Chinois traditionnel](../zht/README.md)** &bull; **[Japonais](../ja/README.md)** &bull;
**[Coréen](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Espagnol](../es/README.md)** &bull; **[Russe](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — Utilisé par [entelecheia](https://github.com/celestia-island/entelecheia) et [shittim-chest](https://github.com/celestia-island/shittim-chest).

Types de messages JSON-RPC 2.0, taxonomie d'agents (14 variantes), environ 100 types de paramètres WebSocket. Liaisons TypeScript auto-générées via `ts-rs`.

## Utilisation

**Rust :**
```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript :**
```bash
pnpm add @celestia-island/arona
```

## Licence

Business Source License 1.1 — utilisation non commerciale sous licence Apache-2.0 ou MIT.
