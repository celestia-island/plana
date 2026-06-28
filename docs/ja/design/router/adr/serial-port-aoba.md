# シリアルポート通信 — aoba による実装

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight は、組み込みデバイス管理および産業用プロトコル（Modbus RTU）プロービングのためにシリアルポートサポートを必要とする。兄弟クレート `aoba` はすでにクロスプラットフォームのシリアルポート列挙、VID/PID/シリアル抽出、および Modbus RTU/TCP マスタ機能を提供している。

## Decision

すべてのシリアルポート操作を `aoba` に委譲する。Evernight の `serial` モジュールは型（`SerialConfig`、`SerialPortInfo`）を定義し、シリアルトランスポート上で `TerminalBackend` を実装し、実際のポート I/O は `aoba` を呼び出す。プロトコル自動検出（Modbus RTU ボーレート/パリティスイープ）も `aoba` に委譲する。

## Consequences

### Positive

- 実績のあるコードを再利用。`aoba` がクロスプラットフォームのエッジケースを処理する

### Negative

- `aoba` への依存が追加される。シリアル機能には `aoba` が利用可能である必要がある
