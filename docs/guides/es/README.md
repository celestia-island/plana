<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Arona logo" width="200"/>


# Arona

**Tipos de protocolo JSON-RPC 2.0 compartidos para la plataforma multiagente Entelecheia**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

## ¿Qué es Arona

Arona define el protocolo de comunicación entre el núcleo de orquestación de agentes de entelecheia y sus interfaces de usuario (TUI, CLI, Web UI, plugins IDE, aplicaciones Tauri). Contiene:

- Tipos de mensajes **JSON-RPC 2.0**
- **Taxonomía de agentes** — enumeración `Agent` (14 variantes)
- **Tipos de parámetros WebSocket** — aproximadamente 100 estructuras que cubren streaming, instantáneas, parches, tareas, configuración de proveedores, base de conocimientos, control de crucero YOLO, señalización WebRTC, árbitro e instantáneas de VM
- **Generación de tipos TypeScript** — todos los tipos se exportan a `bindings/WsTypes.ts` mediante `ts-rs`

### Origen del nombre

Arona (アロナ) — la asistente de IA que coordina misiones y enruta comandos dentro del Cofre de Shittim.

## Uso

**Rust:**

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

## Contribuir

Se aceptan issues y pull requests.

## Licencia

Business Source License 1.1 con doble vía Apache-2.0 / MIT: el uso personal, académico y no comercial está bajo Apache 2.0 o MIT. El uso comercial (alojamiento, reventa, servicios de pago) requiere una licencia BUSL.

Traducciones: [简体中文](../../LICENSE.zhs) · [繁體中文](../../LICENSE.zht) · [Français](../../LICENSE.fr) · [Русский](../../LICENSE.ru) · [العربية](../../LICENSE.ar)
