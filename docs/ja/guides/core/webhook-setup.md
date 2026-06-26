+++
title = "Webhook プラットフォーム設定"
description = """> 現在の webhook レイアウトと統合範囲の説明"""
lang = "ja"
category = "guides"
subcategory = "core"
+++

# Webhook プラットフォーム設定

> 現在の webhook レイアウトと統合範囲の説明

## 概要

リポジトリにはコードホスティングプラットフォームとチャットプラットフォーム向けの webhook 統合が既に含まれていますが、全体としてはまだ移行段階にあり、完全に統一された成熟したソリューションではありません。

現在のディレクトリ構造には以下が共存しています：

- 旧来のプラットフォーム別ディレクトリ：`plugins/github-webhook/github/`、`gitee/`、`gitlab/`、`telegram/`、`qq/`、`lark/`
- 新しい TypeScript 実装：`plugins/github-webhook/ts/`

TypeScript パッケージは現在以下に接続しています：

- GitHub
- Gitee
- GitLab
- 飛書 / Lark
- QQ
- Discord
- Telegram

## 現在できること

- webhook または bot イベントの受信
- WebSocket または HTTP 補助呼び出しを通じてイベントを Scepter に転送
- TypeScript サービスで `/health` ヘルスチェックエンドポイントを提供

## 現在デフォルトで保証できないこと

- すべてのプラットフォームに統一された安定したデプロイソリューション
- 各プラットフォームが完全な issue 駆動 skill chain を形成していること
- すべてのプラットフォーム統合が同じ成熟度に達していること

## TypeScript パッケージ

場所：`plugins/github-webhook/ts/`

開発実行方法：

```bash
cd plugins/github-webhook/ts
npm install
npm run dev
```

本番ビルド方法：

```bash
cd plugins/github-webhook/ts
npm run build
npm start
```

## 主要な環境変数

- `PORT`：webhook サービスポート、デフォルト `8000`
- `SCEPTER_URL`：HTTP 転送アドレス、デフォルト `http://localhost:8424`
- `SCEPTER_WS_URL`：WebSocket 転送アドレス、デフォルト `ws://localhost:8424/ws`

## 使用推奨事項

webhook 機能は「既に存在するが、成熟度が不均衡である」と見なすことができます。特定のプラットフォームに依存する場合は、まず `plugins/github-webhook/` 配下の対応する router または bot の実際の実装を確認してから、安定的な本番利用として説明するかどうかを判断してください。
