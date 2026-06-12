<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Arona logo" width="200"/>


# Arona

**Shared JSON-RPC 2.0 Protocol Types for the Entelecheia Multi-Agent Platform**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> This is the English localization of the [main README](../../README.md).

## What is Arona

Arona defines the **wire protocol** between the entelecheia agent orchestration core and its user-facing shells (TUI, CLI, web UI, IDE plugins, Tauri apps). It contains:

- **JSON-RPC 2.0** message types (`JsonRpcRequest`, `JsonRpcNotification`, `JsonRpcResponse`, `JsonRpcMessage`)
- **Agent taxonomy** — the `Agent` enum (14 variants: HapLotes, SkoPeo, HubRis, KaLos, NeiKos, SkeMma, ApoRia, EleOs, EpieiKeia, OreXis, PhiLia, PoleMos, WebAutomation, ClassicSoftwareEngineering)
- **WebSocket parameter types** — ~100 structs covering streaming, snapshots, patches, tasks, provider configuration, knowledge base, YOLO cruise control, WebRTC signaling, arbiter, and VM snapshots
- **TypeScript type generation** — all types derive `ts-rs::TS` and export to `bindings/WsTypes.ts`

### Named after

Arona (アロナ) — the AI assistant that coordinates missions and routes commands inside the Shittim Chest.

## Usage

**Rust (Cargo.toml):**

```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript (pnpm / npm):**

```bash
pnpm add @celestia-island/arona
```

```ts
import type { Agent, TuiAgentInfo, SkillStage } from "@celestia-island/arona";
```

## Contributing

Issues and pull requests are welcome.

## License

Business Source License 1.1 with Apache-2.0 / MIT dual-path: personal, academic, and non-commercial use is under Apache 2.0 or MIT. Commercial use (hosting, resale, paid services) requires a BUSL license.

Translations: [简体中文](../../LICENSE.zhs) · [繁體中文](../../LICENSE.zht) · [Español](../../LICENSE.es) · [Français](../../LICENSE.fr) · [Русский](../../LICENSE.ru) · [العربية](../../LICENSE.ar)
