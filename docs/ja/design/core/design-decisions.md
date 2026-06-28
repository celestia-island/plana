+++
title = "アーキテクチャ決定記録 (ADR)"
description = """このディレクトリはEntelecheiaの開発中に行われた主要なアーキテクチャ決定を記録します。各ADRは、何が決定されたか、なぜ決定されたか、どのようなトレードオフが検討されたかを説明します。"""
lang = "ja"
category = "design"
subcategory = "core"
+++

# アーキテクチャ決定記録 (ADR)

このディレクトリはEntelecheiaの開発中に行われた主要なアーキテクチャ決定を記録します。各ADRは**何が**決定されたか、**なぜ**決定されたか、どのような**トレードオフ**が検討されたかを説明します。

ADRは[Michael Nygard ADRテンプレート](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)に従います。公開後は不変です — 置き換えられた決定はそれに応じてマークされます。

## 索引

| ADR | タイトル | ステータス |
| --- | --- | --- |
| [ADR-001](en/exec-only-microkernel-tool-surface.md) | Exec-Onlyマイクロカーネルツールサーフェス | 承認済み |
| [ADR-002](en/boa-javascript-engine.md) | 組込みJavaScriptエンジンとしてのBoa | 承認済み |
| [ADR-003](en/postgresql-pgvector-storage.md) | 統合データストレージのためのPostgreSQL + PgVector | 承認済み |
| [ADR-004](en/layered-crate-workspace.md) | 60以上のクレート階層型ワークスペースアーキテクチャ | 承認済み |
| [ADR-005](en/container-sandboxed-agent-execution.md) | COSMOSによるコンテナサンドボックス化エージェント実行 | 承認済み |

## 言語ディレクトリ

| コード | 言語 |
| --- | --- |
| `en/` | 英語（権威） |
| `zhs/` | 簡体字中国語 |
| `zht/` | 繁体字中国語 |
| `ja/` | 日本語 |
| `ko/` | 韓国語 |
| `fr/` | フランス語 |
| `es/` | スペイン語 |
| `ru/` | ロシア語 |
