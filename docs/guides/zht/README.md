<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

# Arona

**Entelecheia 多智能體平台共享 JSON-RPC 2.0 協議類型**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

## Arona 是什麼

Arona 定義了 entelecheia 智能體編排核心與其使用者介面層（TUI、CLI、Web UI、IDE 外掛程式、Tauri 應用程式）之間的通訊協議。包含：

- **JSON-RPC 2.0** 訊息類型
- **智能體分類** — `Agent` 列舉（14 個變體）
- **WebSocket 參數類型** — 約 100 個結構體，涵蓋串流傳輸、快照、修補、任務、提供商配置、知識庫、YOLO 巡航控制、WebRTC 信令、仲裁器、VM 快照
- **TypeScript 類型生成** — 所有類型透過 `ts-rs` 匯出至 `bindings/WsTypes.ts`

### 命名由來

Arona（アロナ）——在什亭之匣內協調任務、路由指令的 AI 助手。

## 用法

**Rust：**

```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript（pnpm / npm）：**

```bash
pnpm add @celestia-island/arona
```

```ts
import type { Agent, TuiAgentInfo, SkillStage } from "@celestia-island/arona";
```

## 參與貢獻

歡迎提交 Issue 和 Pull Request。

## 授權條款

Business Source License 1.1，附帶 Apache-2.0 / MIT 雙路徑：個人、學術和非商業用途適用 Apache 2.0 或 MIT。商業用途（託管、轉售、付費服務）需要 BUSL 授權。

翻譯版本：[简体中文](../../LICENSE.zhs) · [Español](../../LICENSE.es) · [Français](../../LICENSE.fr) · [Русский](../../LICENSE.ru) · [العربية](../../LICENSE.ar)
