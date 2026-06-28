# 接続エントリ URI スキーム

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

接続カタログは、異なるプロトコル接続（SSH、VNC、RDP、シリアル、Docker）を統一的な方法で表現する必要がある。URI スキームは、人間可読かつシリアライズ可能な記述子を提供する。

## Decision

プロトコル固有の URI スキームを使用する:

- `ssh://user@host:port`
- `vnc://host:5900`
- `rdp://host:3389`
- `serial:///dev/ttyUSB0?baud=9600`
- `docker:///var/run/docker.sock?container=name`

`ConnectionEntry` は URI を解析し、スキーム、ホスト、ポート、ユーザー名、パス、クエリパラメータを含む型付き構造体に変換する。カタログはエントリを保持する `ConnectionCategory` ノードのツリー構造である。

## Consequences

### Positive

- 馴染みのある URI 形式。容易にシリアライズ可能。接続のコピー＆ペースト共有をサポート

### Negative

- 一部のプロトコルは URI にきれいにマッピングできない（例: Kubernetes コンテキスト）
- クエリ文字列パラメータは非構造的である
