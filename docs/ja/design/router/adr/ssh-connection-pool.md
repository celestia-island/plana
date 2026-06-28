# SSH 接続プール

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

元のコードは操作（ファイル一覧、コマンド実行）ごとに新しい SSH 接続を開いていた。これは高コストであり（呼び出しごとに TCP + 鍵交換 + 認証）、複数の並行操作を伴う対話的な使用にはスケールしない。

## Decision

`(host, port, username)` でキー付けされた `SshConnectionPool` を実装する。接続は最初の要求時に遅延確立され、操作間で再利用される。`PooledSshClient` は共有の `Arc<Mutex<SshSession>>` をラップする。定期的なヘルスチェックが死んだ接続を除去する。

## Consequences

### Positive

- 繰り返し操作のレイテンシが劇的に低減
- 単一接続上での並行シェル + ファイル + 端末セッションが可能

### Negative

- プールは接続の有効期限と再接続を処理する必要がある
- 長時間の接続はファイアウォールによって切断される可能性がある
