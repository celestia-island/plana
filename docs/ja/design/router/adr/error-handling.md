# エラーハンドリング — thiserror とクレート Result

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight は、全モジュール（画面、SSH、ハードウェア、ネットワーク、トンネリング）にわたる統一されたエラー型を必要とする。ライブラリは、内部のエラーハンドリングの人間工学性を保ちつつ、クリーンなエラー API を公開しなければならない。

## Decision

`thiserror` を使用して、バリアントごとに `#[error(...)]` 表示実装を持つ `EvernightError` 列挙型を導出する。クレート全体の結果型として `pub type Result<T> = std::result::Result<T, EvernightError>` を定義する。各バリアントは、内部の依存関係の詳細を漏洩させないために、外部のエラー型をラップするのではなく、ドメイン固有のコンテキスト（`ScreenCapture`、`Ssh`、`Tunnel` など）を `String` としてキャプチャする。

## Consequences

### Positive

- 安定した公開エラー API。利用者は内部を知らなくても列挙型バリアントでマッチできる

### Negative

- `String` 変換による一部の情報損失
- 構造化されたエラーチェーンがない
