# SSH 設定パーサー

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

ユーザーは、`ssh` コマンドと同様に、`evernight` がホストエイリアス、ジャンプホスト、鍵ファイルのために `~/.ssh/config` を読み取ることを期待する。これがないと、ユーザーは毎回すべての接続パラメータを繰り返し指定する必要がある。

## Decision

標準的なディレクティブ（`Host`、`HostName`、`User`、`Port`、`IdentityFile`、`ProxyJump`、`ForwardAgent`、`ServerAliveInterval` など）を処理するピュア Rust SSH 設定パーサーを実装する。`Host` エントリに対するグロブパターンマッチング。設定ベースの透過的な接続のために接続プールと統合する。

## Consequences

### Positive

- ドロップインの SSH 互換性。設定からのジャンプホスト解決

### Negative

- OpenSSH 設定構文の変更を追跡する必要がある
- `Match` ディレクティブのサポートは複雑である
