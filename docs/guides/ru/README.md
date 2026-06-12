<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Arona logo" width="200"/>


# Arona

**Общие типы протокола JSON-RPC 2.0 для мультиагентной платформы Entelecheia**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

## Что такое Arona

Arona определяет протокол связи между ядром оркестрации агентов entelecheia и его пользовательскими интерфейсами (TUI, CLI, Web UI, плагины IDE, приложения Tauri). Содержит:

- Типы сообщений **JSON-RPC 2.0**
- **Таксономия агентов** — перечисление `Agent` (14 вариантов)
- **Типы параметров WebSocket** — около 100 структур, охватывающих потоковую передачу, снимки, патчи, задачи, конфигурацию провайдеров, базу знаний, круиз-контроль YOLO, сигнализацию WebRTC, арбитр и снимки виртуальных машин
- **Генерация типов TypeScript** — все типы экспортируются в `bindings/WsTypes.ts` через `ts-rs`

### Происхождение названия

Arona (アロナ) — ИИ-ассистент, координирующий миссии и маршрутизирующий команды внутри Скинии Ковчега.

## Использование

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

## Участие

Приветствуются issue и pull request'ы.

## Лицензия

Business Source License 1.1 с двойным путём Apache-2.0 / MIT: для личного, академического и некоммерческого использования применяется Apache 2.0 или MIT. Коммерческое использование (хостинг, перепродажа, платные услуги) требует лицензии BUSL.

Переводы: [简体中文](../../LICENSE.zhs) · [繁體中文](../../LICENSE.zht) · [Español](../../LICENSE.es) · [Français](../../LICENSE.fr) · [العربية](../../LICENSE.ar)
