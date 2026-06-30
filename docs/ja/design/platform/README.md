+++
title = "プラットフォーム設計ドキュメント"
description = """クロスプロジェクト (プラットフォームレベル) の設計ドキュメント。プロジェクト別の core/webui/router サブカテゴリとは異なり、ここにある文書は三つのプロジェクト (entelecheia、shittim-chest、evernight) すべてにまたがる関心事を扱う——例えば三者が共有する統合スーパビジョン、ローリングアップデート、レプリケーションアーキテクチャなど。"""
lang = "ja"
category = "design"
subcategory = "platform"
+++

# プラットフォーム設計ドキュメント

> **スコープ。** これらの文書は*プラットフォームレベル*であり、`core`
> (entelecheia)、`webui` (shittim-chest)、`router` (evernight) を横断する。
> プロジェクトごとの設計はそれぞれのサブカテゴリに置かれている。

## インデックス

| 文書 | 概要 |
| --- | --- |
| [スーパビジョン・ローリングアップデート・レプリケーション](supervision-and-rolling-update.md) | 三プロジェクトすべてが共有する単一のスーパビジョンツリーバックボーン: 統一されたシグナル/ドレインセマンティクス、ゼロダウンタイムの引き継ぎのための systemd socket activation、差し替え可能な coordination lock trait、そして同じ Worker + Supervisor プリミティブの上に構築された二つのフォールトトレランス戦略 (Replica = ロードバランシング ⊃ ローリングアップデート; Leader/Follower = エッジ HA)。 |

## 言語ディレクトリ

| コード | 言語 |
| --- | --- |
| `en/` | 英語 (権威) |
| `zhs/` | 簡体字中国語 |
| `zht/` | 繁体字中国語 |
| `ja/` | 日本語 |
| `ko/` | 韓国語 |
| `fr/` | フランス語 |
| `es/` | スペイン語 |
| `ru/` | ロシア語 |
