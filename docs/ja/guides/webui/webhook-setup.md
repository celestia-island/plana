+++
title = "Webhook設定ガイド"
description = """> 対象: 外部サービスをshittim-chestと統合する管理者。"""
lang = "ja"
category = "guides"
subcategory = "webui"
+++

# Webhook設定ガイド

> **対象**: 外部サービスをshittim-chestと統合する管理者。
> **最終更新日**: 2026-05-25

## 概要

Webhookを使用すると、外部サービス（GitHub、GitLab、Gitee）がリアルタイムイベントをshittim-chestに送信できます。イベントは検証、解析され、適切なエージェントにディスパッチするscepterに転送されます。

```text
外部サービス → shittim_chest → scepter → エージェント
```

shittim_chestは、ネイティブにサポートされていないサービス用のカスタムwebhookエンドポイントもサポートしています。

## GitHub Webhook設定

### ステップ1: 環境設定

`.env`にwebhookシークレットを設定します:

```bash
WEBHOOK_GITHUB_SECRET=your-hmac-secret-here
WEBHOOK_PUBLIC_URL=https://your-domain.com
```

強力なシークレットを生成:

```bash
openssl rand -hex 32
```

### ステップ2: GitHubでWebhookを作成

1. リポジトリに移動 → **Settings** → **Webhooks** → **Add webhook**
1. **Payload URL**を`https://your-domain.com/api/webhook/github`に設定
1. **Content type**を`application/json`に設定
1. **Secret**を`WEBHOOK_GITHUB_SECRET`と同じ値に設定
1. イベントを選択: `push`、`pull_request`、`issues`、`issue_comment`
1. **Active**がチェックされていることを確認
1. **Add webhook**をクリック

### ステップ3: 検証

GitHubはすぐに`ping`イベントを送信します。**Recent Deliveries**タブを確認し、`200`レスポンスを確認してください。

## GitLab Webhook設定

### ステップ1: 環境設定

```bash
WEBHOOK_GITLAB_SECRET=your-gitlab-secret-token
```

### ステップ2: GitLabでWebhookを作成

1. プロジェクトに移動 → **Settings** → **Webhooks**
1. **URL**を`https://your-domain.com/api/webhook/gitlab`に設定
1. **Secret token**を`WEBHOOK_GITLAB_SECRET`と同じ値に設定
1. トリガーを選択: `Push events`、`Merge request events`、`Issue events`
1. **Enable SSL verification**がチェックされていることを確認（HTTPSの場合）
1. **Add webhook**をクリック

### ステップ3: 検証

GitLabの**Test**ボタンを使用してテストイベントを送信します。配信が成功することを確認してください。

## Gitee Webhook設定

Gitee（码云）のwebhookもサポートされています。

### ステップ1: 環境設定

GiteeはHMAC検証に同じ`WEBHOOK_GITLAB_SECRET`を使用します（トークンフォールバック付き）。パスワードベースの認証を使用する場合は、代わりに`WEBHOOK_GITEE_PASSWORD`を設定します。

### ステップ2: GiteeでWebhookを作成

1. リポジトリに移動 → **Management** → **Webhooks**
1. **URL**を`https://your-domain.com/api/webhook/gitee`に設定
1. **Password/Signing Key**を同じシークレットに設定
1. イベントを選択: `Push`、`Pull Request`、`Issues`
1. **Add**をクリック

## カスタムWebhook

shittim_chestは`/api/webhook/custom/{name}`で汎用カスタムwebhookエンドポイントをサポートしています。カスタムwebhookソースを追加するには:

1. `.env`に`WEBHOOK_PUBLIC_URL`を設定
1. 外部サービスが`https://your-domain.com/api/webhook/custom/{name}`にPOSTするように設定
1. イベントはwebhook名をイベントソースとしてscepterに転送されます

コードレベルで新しいwebhookプロバイダーを統合するには:

1. `packages/core/src/webhook.rs`にハンドラを追加
1. 新しいプロバイダー用のHMACまたはトークン検証を実装
1. カスタムイベント形式を解析し、Unixソケット経由でscepterに転送

## IPホワイトリスト

shittim_chestは、不明な発信元からのリクエストを拒否するためにwebhookソースのIPホワイトリストをサポートしています:

```bash
# .env
WEBHOOK_IP_WHITELIST=140.82.112.0/20,192.30.252.0/22  # GitHub IP
```

各webhookプロバイダーのCIDR範囲を設定します。ホワイトリスト外のIPからのリクエストは拒否されます。

## イベントタイプ

サポートされるイベントとscepterトリガーへのマッピング:

| ソース | イベント | scepter `event_type` |
| --- | --- | --- |
| GitHub | `push` | `github.push` |
| GitHub | `pull_request` | `github.pull_request` |
| GitHub | `issues` | `github.issues` |
| GitHub | `issue_comment` | `github.issue_comment` |
| GitLab | `push` | `gitlab.push` |
| GitLab | `merge_request` | `gitlab.merge_request` |
| GitLab | `issues` | `gitlab.issues` |
| Gitee | `push` | `gitee.push` |
| Gitee | `pull_request` | `gitee.pull_request` |
| Gitee | `issues` | `gitee.issues` |

## 配信ログ

shittim_chestはwebhookイベントの配信ログを保持します。重複配信はLRUキャッシュ（最大10,000配信ID）を使用して検出されます。配信ログには以下からアクセスできます:

- **REST API**: `GET /api/webhook/deliveries`
- 管理パネル: **Webhooks** → **Delivery Log**

## セキュリティ

すべてのwebhookは署名検証を通過する必要があります:

- **GitHub**: `X-Hub-Signature-256`ヘッダーを使用。`WEBHOOK_GITHUB_SECRET`に対して検証。
- **GitLab**: `X-Gitlab-Token`ヘッダーを使用。`WEBHOOK_GITLAB_SECRET`に対して検証。
- **Gitee**: トークンフォールバック付きHMAC-SHA256署名を使用。

有効な署名のないリクエストは`401 Unauthorized`で拒否されます。webhookシークレットをクライアント側のコードやログに決して露出させないでください。

## テスト

管理パネルを使用してwebhook統合をテストします:

1. 管理パネルにログイン（デフォルト`:3000`）
1. サイドバーの**Webhooks**に移動
1. 配信ログと設定を表示
1. 外部サービスのテスト機能を通じてエンドポイントをテスト

curlを使用して手動でテストすることもできます:

```bash
curl -X POST https://your-domain.com/api/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=<computed-hmac>" \
  -d '{"action":"push","ref":"refs/heads/main"}'
```

## トラブルシューティング

### 401 Unauthorized

**原因**: HMAC署名の不一致、またはIPがホワイトリストにない。
**修正**: `.env`のシークレットがソースプラットフォームで設定されたシークレットと一致することを確認。末尾の空白やエンコーディングの問題を確認。IPホワイトリスト設定を検証。

### 502 Bad Gateway

**原因**: scepterに到達不能。
**修正**: `.env`の`ENTELECHEIA_SCEPTER_URL`と`ENTELECHEIA_TUI_SOCK`を検証。scepterインスタンスが実行中で、Unixソケットパスにアクセス可能であることを確認。

### イベントがエージェントに届かない

**原因**: イベントタイプがマッピングされていない、またはエージェントが処理するように設定されていない。
**修正**: 解析された`event_type`のバックエンドログを確認。ターゲットエージェントがそのイベントのハンドラを登録していることを検証。APIまたは管理パネル経由で配信ログを確認。

### 重複配信

**原因**: タイムアウトにより外部サービスが再試行している。shittim_chestはLRUキャッシュを通じて自動的に重複を検出。
**修正**: 有効な再試行がブロックされている場合、配信IDキャッシュサイズを増やす。shittim_chestがサービスのタイムアウトウィンドウ内で応答することを確認（GitHub: 10秒）。
