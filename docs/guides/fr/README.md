<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Arona logo" width="200"/>


# Arona

**Types de protocole JSON-RPC 2.0 partagés pour la plateforme multi-agents Entelecheia**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

## Qu'est-ce qu'Arona

Arona définit le protocole de communication entre le noyau d'orchestration d'agents entelecheia et ses interfaces utilisateur (TUI, CLI, Web UI, plugins IDE, applications Tauri). Contient :

- Types de messages **JSON-RPC 2.0**
- **Taxonomie des agents** — énumération `Agent` (14 variantes)
- **Types de paramètres WebSocket** — environ 100 structures couvrant le streaming, les instantanés, les correctifs, les tâches, la configuration des fournisseurs, la base de connaissances, le contrôle de croisière YOLO, la signalisation WebRTC, l'arbitre et les instantanés VM
- **Génération de types TypeScript** — tous les types sont exportés vers `bindings/WsTypes.ts` via `ts-rs`

### Origine du nom

Arona (アロナ) — l'assistante IA qui coordonne les missions et achemine les commandes dans le Coffre de Shittim.

## Utilisation

**Rust :**

```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript (pnpm / npm) :**

```bash
pnpm add @celestia-island/arona
```

```ts
import type { Agent, TuiAgentInfo, SkillStage } from "@celestia-island/arona";
```

## Contribuer

Les issues et pull requests sont les bienvenues.

## Licence

Business Source License 1.1 avec double voie Apache-2.0 / MIT : l'utilisation personnelle, académique et non commerciale est sous Apache 2.0 ou MIT. L'utilisation commerciale (hébergement, revente, services payants) nécessite une licence BUSL.

Traductions : [简体中文](../../LICENSE.zhs) · [繁體中文](../../LICENSE.zht) · [Español](../../LICENSE.es) · [Русский](../../LICENSE.ru) · [العربية](../../LICENSE.ar)
