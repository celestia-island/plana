<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

# Arona

**Entelecheia マルチエージェントプラットフォーム向け 共有 JSON-RPC 2.0 プロトコル型**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

## Arona とは

Arona は、entelecheia エージェントオーケストレーションコアとユーザーインターフェース層（TUI、CLI、Web UI、IDE プラグイン、Tauri アプリ）間の通信プロトコルを定義します。内容：

- **JSON-RPC 2.0** メッセージ型
- **エージェント分類** — `Agent` 列挙型（14 バリアント）
- **WebSocket パラメータ型** — ストリーミング、スナップショット、パッチ、タスク、プロバイダ設定、ナレッジベース、YOLO クルーズ制御、WebRTC シグナリング、アービタ、VM スナップショットをカバーする約 100 の構造体
- **TypeScript 型生成** — 全型が `ts-rs` により `bindings/WsTypes.ts` へエクスポート

### 名前の由来

Arona（アロナ）——シッティムチェスト内でミッションを調整し、コマンドをルーティングする AI アシスタント。

## 使い方

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

## コントリビュート

Issue と Pull Request を歓迎します。

## ライセンス

Business Source License 1.1、Apache-2.0 / MIT デュアルパス：個人、学術、非商用利用は Apache 2.0 または MIT が適用されます。商用利用（ホスティング、再販、有料サービス）には BUSL ライセンスが必要です。

翻訳：[简体中文](../../LICENSE.zhs) · [繁體中文](../../LICENSE.zht) · [Español](../../LICENSE.es) · [Français](../../LICENSE.fr) · [العربية](../../LICENSE.ar)
