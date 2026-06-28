+++
title = "アーキテクチャ詳細"
description = """> 対象: shittim-chestが内部的にどのように動作するかを理解する必要がある開発者。"""
lang = "ja"
category = "guides"
subcategory = "webui"
+++

# アーキテクチャ詳細

> **対象**: shittim-chestが内部的にどのように動作するかを理解する必要がある開発者。
> **最終更新日**: 2026-05-25

## プロジェクト概要

shittim-chestは、Rustベースのマルチエージェントコラボレーションプラットフォームである[entelecheia](https://github.com/celestia-island/entelecheia)の**ユーザー向けシェル**です。境界は意図的に設定されています:

- **entelecheia**はエージェントオーケストレーション（scepter、13エージェント、Cosmos/IEPLランタイム）、アイデンティティ、権限を所有します。
- **shittim-chest**はユーザー認証、セッション管理、チャットデータ、LLMプロバイダー設定、フロントエンドプレゼンテーション、およびscepterへのプロキシブリッジを所有します。

両者はJWT認証されたHTTPおよびWebSocketを通じて通信します。shittim-chestがエージェント操作のためにentelecheiaのデータベースに直接アクセスすることはありません。

## バックエンドスタック

### Axumルーター

コアバックエンド（`packages/core`）はAxum 0.8アプリケーションです。ルーターは以下のモジュールグループをマウントします:

```text
/                   → ヘルスチェック
/api/auth/*         → AuthService（ログイン、登録、GitHub OAuth、リフレッシュ、ログアウト）
/api/chat/*         → ChatService（会話、メッセージ、SSE/WSストリーミング、検索、エクスポート）
/api/providers/*    → ProviderService（LLMプロバイダーCRUD、APIキー暗号化、テスト）
/api/generation/*   → GenerationService（画像生成）
/api/devices/*      → DeviceService（リモートデバイス一覧、セッション、シグナリング）
/api/webhook/*      → WebhookService（GitHub、GitLab、Gitee、カスタム。HMAC検証）
/api/proxy/*        → ProxyService（scepterへのHTTPリバースプロキシ + WebSocketブリッジ）
/static/*           → SPA静的ホスティング（プロダクションのみ）
```

### SeaORM + PostgreSQL

データベースアクセスはPostgreSQLを用いたSeaORM 1.xを使用します。`shittim_chest_db`は以下を保存します:

- ユーザー認証: パスワードハッシュ（argon2）、セッション、リフレッシュトークン、APIキー、OAuth接続
- チャットデータ: 会話、メッセージ
- LLMプロバイダー設定（APIキーはAES-256-GCMで保存時に暗号化）
- リモートデバイスレコードとデバイスセッション
- マルチプラットフォームメッセージング用チャネル設定
- Webhook配信ログ

5つのマイグレーションと25のエンティティモデルが`packages/core/src/{migration,entity}/`にあります。

### JWT認証

shittim_chestは`{ sub: user_id, groups: [...] }`を含むJWTを発行します。JWT秘密鍵はscepterと共有されるため、両方のサービスが独立してトークンを検証できます。アクセストークンは1時間で期限切れ、リフレッシュトークンは7日間で、使用ごとにローテーションされます。

## 独立したLLM機能

shittim-chestはentelecheiaから独立して動作する独自のLLMルーティング層を持っています:

- **LlmRouter**: 優先度ベースの選択とフォールバックを持つマルチプロバイダールーター
- **プロバイダー管理**: LLMプロバイダーの追加/編集/削除のためのCRUDエンドポイント
- **APIキー暗号化**: プロバイダーAPIキーはAES-256-GCMで保存時に暗号化
- **OpenAI互換**: 任意のOpenAI互換API（DeepSeek、OpenAI、ローカルモデルなど）で動作
- **デュアルストリーミング**: チャット応答用のSSE（Server-Sent Events）およびWebSocketストリーミング

これは、shittim-chestがentelecheiaなしでスタンドアロンのチャットアプリケーションとして実行できるか、プロキシ層を通じてentelecheiaエージェントを使用できることを意味します。

## 認証フロー

### ログインシーケンス

```text
ユーザー → shittim_chest: POST /api/auth/login { username, password }
shittim_chest → shittim_chest_db: SELECT user WHERE username = ? (argon2ハッシュ検証)
shittim_chest → scepter: GET /api/user/{id}/permissions
scepter → entelecheia_db: グループ + 権限をクエリ
scepter → shittim_chest: { groups: [...], permissions: {...} }
shittim_chest → ユーザー: { access_token, refresh_token }
shittim_chest: セッション保存 + RBACキャッシュ
```

### GitHub OAuth

```text
ユーザー → shittim_chest: GET /api/auth/github
shittim_chest → ユーザー: 302 GitHub OAuthにリダイレクト
ユーザー → GitHub: 認可
GitHub → shittim_chest: GET /api/auth/github/callback?code=...
shittim_chest → GitHub: コードをアクセストークンと交換
shittim_chest → GitHub: GET /user (ユーザー情報取得)
shittim_chest → shittim_chest_db: INSERT/UPDATE oauth_connections
shittim_chest → ユーザー: { access_token, refresh_token } (新規ユーザーの場合は自動作成)
```

## チャットアーキテクチャ

### メッセージフロー（スタンドアロンLLM）

```text
ユーザー → POST /api/chat/conversations/:id/messages
shittim_chest: JWT検証、会話読み込み
shittim_chest → LlmRouter: 最適なプロバイダーにリクエストをルーティング
LlmRouter → LLMプロバイダー: POST chat/completions (ストリーミング)
LLMプロバイダー → LlmRouter: SSEストリーム
LlmRouter → ユーザー: SSE/WSストリーム（トークンが到着次第）
shittim_chest: shittim_chest_dbにメッセージを永続化
```

### SSE vs WebSocketストリーミング

- **SSE** (`/api/chat/stream`): シンプルなHTTPストリーミング、プロキシを通過、自動再接続
- **WebSocket** (`/ws/chat/stream`): 双方向、キャンセルとリアルタイム対話をサポート

## プロキシアーキテクチャ

`/api/proxy/*`エンドポイントは認証済みリクエストをscepterに転送します:

1. ブラウザがJWT付きで`ws://shittim-chest:80/api/proxy/chat`を開く
1. shittim_chestがJWTを検証し、JWTを転送してscepterへの接続を開く
1. ブラウザとscepter間の双方向メッセージ転送
1. shittim_chestがレート制限を強制し、使用量をログに記録し、接続ライフサイクルを管理

## Webhookパイプライン

外部サービスからのWebhookは`/api/webhook/*`を通じて入ります:

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC検証 → イベント解析 → Unixソケット経由でscepterに転送
```

サポートされるソース: GitHub（HMAC-SHA256）、GitLab（トークン）、Gitee（HMAC + トークンフォールバック）、および汎用`/api/webhook/custom/{name}`エンドポイント。機能:

- 重複配信検出（LRUキャッシュ、10,000 ID）
- 一覧API付き配信ログ
- webhookソース用IPホワイトリスト

## リモートデバイス管理

リモートデバイスはシグナリングリレーを通じて管理されます:

```text
ブラウザ（webui） → WS /api/devices/stream → shittim_chest（シグナルリレー） → Unixソケット → entelecheia/polemos
```

機能:

- REST経由のデバイス一覧およびセッションCRUD
- WebRTCシグナリング（SDPオファー/アンサー、ICE候補）
- ターミナルリレー（WebSocketからxterm.jsへ）
- デスクトップフレームリレー
- SFTPファイルブラウザバックエンド

shittim-chestがリモートデバイスに直接接続することは決してありません — すべてのデータはentelecheiaのpolemosエージェントを通じて流れます。

## データ所有権

### shittim_chest_db

| データ | テーブル | 根拠 |
| --- | --- | --- |
| パスワードハッシュ（argon2） | `auth_users` | プレゼンテーション層がログインフローを所有 |
| アクティブセッション、リフレッシュトークン | `sessions` | セッション管理はフロントエンドの関心事 |
| 暗号化APIキー | `api_keys` | APIキー発行はユーザー向け |
| OAuth接続 | `oauth_connections` | サードパーティ認証バインディングはユーザー向け |
| 会話、メッセージ | `conversations`、`messages` | チャットデータはユーザー向け |
| LLMプロバイダー設定 | `llm_providers` | プロバイダー管理はユーザー向け（キーは暗号化） |
| リモートデバイスレコード | `remote_devices`、`device_sessions` | デバイス追跡はユーザー向け |
| チャネル設定 | `channel_configs` など | マルチプラットフォーム設定はユーザー向け |

### entelecheia_db

| データ | 根拠 |
| --- | --- |
| ユーザーID、グループ、ロール割り当て | コアが権限を強制 |
| GroupPermissions（プロバイダークォータ、エージェントホワイトリスト） | エージェントレベルのポリシーはエージェントと共に存在 |
| エージェント設定、Cosmos/IEPL状態 | オーケストレーションデータはコアに属する |

## デュアルフロントエンド戦略

### フェーズ1: Vue 3（現在）

| パッケージ | 技術 | ポート | 目的 |
| --- | --- | --- | --- |
| `webui` | Vue 3 + Vite + Pinia (TSX) | `:3000（共有）` | 統合webui: チャット、画像生成、デバイス、管理（プロバイダー、エージェント、RBAC、webhook） |

### フェーズ2: Rust WASM（将来）

| パッケージ | 技術 | 目的 |
| --- | --- | --- |
| `webui` | Rust → WASM (Tairitsu) | 長期的な統合webui（チャット + 管理） |

レガシーフロントエンドは生きた仕様として機能します。移行期間中、両方のバージョンが並行して実行され、同一のユーザー操作は同一の結果を生成しなければなりません。

## リバースプロキシデプロイメントモード

shittim-chestは、`.env`の`SHITTIM_CHEST_PROXY_MODE`で制御される3つのリバースプロキシモードをサポートします。

### モード1: None（直接）

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=none   # または未設定
```

コアサーバーが`SHITTIM_CHEST_HOST:SHITTIM_CHEST_PORT`（デフォルト`0.0.0.0:80`）に直接バインドします。TLSなし、リバースプロキシコンテナなし。以下に適しています:

- ローカル開発
- 既存のリバースプロキシの背後（Cloudflare Tunnel、AWS ALB、Traefikラベル）
- 別のサービスがTLS終端を処理するDockerネットワーク

### モード2: Caddy自動

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_DOMAIN=app.example.com
```

CLIが以下のことを行う`shittim-chest-caddy`コンテナ（イメージ`caddy:2`）を作成します:

1. ポート80/443でリッスン（`SHITTIM_CHEST_PROXY_HTTP_PORT` / `SHITTIM_CHEST_PROXY_HTTPS_PORT`で設定可能）
1. Let's Encrypt（Caddyの組み込みACME）を通じてTLS証明書を自動プロビジョニング
1. すべてのリクエストをDockerネットワーク上のコアバックエンドにプロキシ

Caddyfileは不要です — CLIが自動生成します。ドメインはパブリックDNSがホストを指している必要があります。

### モード3: Caddyカスタム

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/caddy/Caddyfile
SHITTIM_CHEST_PROXY_EXTRA_VOLUMES=/etc/letsencrypt:/etc/letsencrypt
```

同じCaddyコンテナですが、独自のCaddyfile（ホストからマウント）を提供します。以下が必要な場合に使用します:

- 複数の仮想ホスト
- カスタムTLS証明書パス
- 追加のミドルウェア（基本認証、レート制限など）
- APIと共に静的ファイルを提供

### モード4: Nginxカスタム

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=nginx
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/nginx/conf.d/default.conf
```

設定ファイル付きの`nginx:bookworm`コンテナを作成します。TLS証明書は自身で管理します。Nginxが標準の環境に適しています。

### コンテナライフサイクル

すべてのプロキシコンテナはDocker API（`bollard`）を通じてCLIによって管理されます:

| コマンド | 動作 |
| --- | --- |
| `just dev` / `chest up` | `PROXY_MODE`が設定されている場合、プロキシコンテナを作成/起動 |
| `just dev-stop` / `chest down` | プロキシコンテナを停止および削除 |
| コンテナが既に実行中 | 既存のコンテナを再利用（冪等） |

プロキシコンテナはコアバックエンドと同じDockerネットワークに参加するため、内部ホスト名（`core`または`shittim-chest`）を通じてバックエンドに到達します。
