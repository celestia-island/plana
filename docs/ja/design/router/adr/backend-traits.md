+++
title = "TerminalBackend / ViewportBackend / FileBackend トレイト抽象化"
description = """アーキテクチャ意思決定記録 —— TerminalBackend / ViewportBackend / FileBackend トレイト抽象化。"""
lang = "ja"
category = "design"
subcategory = "router"
+++

# TerminalBackend / ViewportBackend / FileBackend トレイト抽象化

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight は、フロントエンド（CLI、TUI、GUI）が任意のプロトコルを統一的に利用できるように、多相的なバックエンドインターフェースを必要とする。共有トレイトがなければ、各フロントエンドは端末、グラフィカル表示、ファイル操作のためにプロトコル固有のコードパスを実装する必要がある。

## Decision

クレートルートに3つのオブジェクトセーフな非同期トレイトを定義する（常に利用可能、フィーチャフラグ不要）:

- **`TerminalBackend`** — `read` / `write` / `resize` / `close`
- **`ViewportBackend`** — `render` / `input` / `clipboard` / `close`
- **`FileBackend`** — `list` / `stat` / `get` / `put` / `rm` / `mkdir` / `rename`

各プロトコルバックエンドは対応するトレイトを実装する。フロントエンドは `Box<dyn TerminalBackend>` などを利用する。

## Consequences

### Positive

- フロントエンドはプロトコル非依存。新しいバックエンド（例: RDP）をフロントエンドの変更なしに追加できる

### Negative

- 非同期トレイトオブジェクトには `Box::pin` のオーバーヘッドが伴う
- トレイト設計は安定していなければならない。変更するとすべての実装者が破壊される
