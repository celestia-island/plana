+++
title = "SSH バックエンド — russh と共有接続"
description = """アーキテクチャ意思決定記録 —— SSH バックエンド — russh と共有接続。"""
lang = "ja"
category = "design"
subcategory = "router"
+++

# SSH バックエンド — russh と共有接続

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight は、リモートシェル、ファイル操作、端末アクセスのために SSH を必要とする。当初のコードベースには、接続ロジックが重複した 3 つの別々の SSH ハンドラ実装（`FileHandler`、`SshHandler`、`TerminalHandler`）があった。本プロジェクトはピュア Rust を要求する（`Command::new("ssh")` によるシェル呼び出し禁止）。

## Decision

SSH バックエンドとして `russh`（ピュア Rust SSH-2 実装）を使用する。すべての SSH ハンドラ実装を `remote/connection.rs` 内の単一の `DefaultSshHandler` に統合し、共有の `connect_session()` 関数を使用する。すべての SSH 操作（シェル、ファイル、端末）はこの接続抽象化を共有する。

## Consequences

### Positive

- SSH 認証ロジックの単一の信頼できる情報源
- 将来的な接続プーリングの追加が容易

### Negative

- `russh` はエッジケースにおいて OpenSSH に遅れを取る可能性がある
- 組み込みの SSH エージェントフォワーディングは未対応
