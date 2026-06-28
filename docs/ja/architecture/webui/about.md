+++
title = "Shittim Chestについて"
description = """バージョン 0.1.0"""
lang = "ja"
category = "architecture"
subcategory = "webui"
+++

# Shittim Chest（什亭之匣）

**バージョン 0.1.0**

Shittim Chestは、[entelecheia](https://github.com/celestia-island/entelecheia)マルチエージェントコラボレーションプラットフォームのユーザー向けシェルであり、RustとVue 3で構築されています。

## アーキテクチャ

Shittim Chestは、完全なユーザー体験を提供するために連携する複数のコンポーネントで構成されています：

- **arona** — 現在ご利用いただいているチャットUI。ストリーミング応答、画像生成、エージェント状態監視、シンキングウィンドウ、リモートデバイスビューアー、多言語サポートを備えています。
- **`shittim_chest`** — 認証（JWT + OAuth）、独立したLLMルーティング、チャットAPI、画像生成、webhookイングレス、scepterプロキシ、リモートデバイスシグナリングを処理する統合Rust + Axumバックエンド。

## Entelecheiaとの関係

[entelecheia](https://github.com/celestia-island/entelecheia)は、中核的なマルチエージェントオーケストレーションエンジンです。エージェントランタイム（scepter、13の特化エージェント、Cosmos/IEPLランタイム）を提供します。Shittim Chestは、ユーザーが直接操作するすべて — アイデンティティ、プレゼンテーション、コミュニケーション — を処理します。

2つのプロジェクトは設計上分離されています：entelecheiaはエージェントオーケストレーションを管理し、shittim-chestはユーザーアイデンティティとプレゼンテーションを管理します。両者はJWT認証されたHTTP/WebSocketを通じて通信します。ログイン認証情報はshittim_chest_dbに保存され、権限とアイデンティティデータはentelecheia_dbに保存されます。この分離により、フロントエンドシェルはエージェントコアから独立して進化できます。

## Hikariとの関係

[hikari](https://github.com/celestia-island/hikari)は、Celestia Islandエコシステムのゲートウェイおよびルーティング層です。すべての外部トラフィックのエントリポイントとして機能し、shittim-chest、entelecheia、その他サービス間のリクエストルーティング、ロードバランシング、APIゲートウェイ機能を処理します。

## Tairitsuとの関係

[tairitsu](https://github.com/celestia-island/tairitsu)は、Celestia Islandエコシステムのクロスプラットフォームネイティブアプリケーションフレームワークです。aronaをネイティブアプリケーションとしてラップするTauriベースのデスクトップおよびモバイルクライアントと、開発ワークフローを支えるブラウザ自動化およびテストインフラストラクチャを提供します。

## ライセンス

Shittim Chestは**Business Source License 1.1 (BSL-1.1)**の下でライセンスされています。

**非商用利用** — 内部運用、学術研究、教育、個人学習、評価、政府および公共サービス、教育利用を含む — において、付与される権利は**Synthetic Source License 1.0 (SySL-1.0)**（「無料利用ライセンス」）と同等です。これらの目的のために、自由にソフトウェアを使用、研究、改変、実行することができます。

**商用利用** — 第三者へのホスティングサービスとしての提供、スタンドアロン製品としての再頒布、商用製品の中核コンポーネントとしての使用など — には、許諾者からの別途商用ライセンスが必要です。

詳細については[ライセンス全文](https://github.com/celestia-island/shittim-chest/blob/main/LICENSE)をご参照ください。

---

[Celestia Island](https://github.com/celestia-island)により❤を込めて構築。
