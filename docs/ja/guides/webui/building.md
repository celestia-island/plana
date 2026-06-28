+++
title = "ビルドと開発ガイド"
description = """> 対象: ローカルのshittim-chest開発環境をセットアップするコントリビューター。"""
lang = "ja"
category = "guides"
subcategory = "webui"
+++

# ビルドと開発ガイド

> **対象**: ローカルのshittim-chest開発環境をセットアップするコントリビューター。
> **最終更新日**: 2026-05-25

## 前提条件

| ツール | 最小バージョン | 備考 |
| --- | --- | --- |
| Rust | 1.85以上 | Edition 2024が必要。<https://rustup.rs>からインストール |
| Node.js | 20以上 | LTS推奨 |
| pnpm | 9以上 | `corepack enable && corepack prepare pnpm@latest --activate` |
| just | 最新 | コマンドランナー。`cargo install just` |
| PostgreSQL | 18以上 | 認証 + チャットデータ用shittim_chest_db |
| entelecheia scepter | 任意 | プロキシ/デバイス機能に必要。スタンドアロンチャットには不要 |

すべてを検証:

```bash
rustc --version    # >= 1.85
node --version     # >= 20
pnpm --version     # >= 9
just --version
psql --version     # >= 18
```

## クローンとブートストラップ

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
```

## 環境変数

クローン後に`.env`を編集してください。すべての変数はインラインで文書化されています。以下は概要です。

### サーバー

| 変数 | デフォルト | 目的 |
| --- | --- | --- |
| `SHITTIM_CHEST_HOST` | `0.0.0.0` | リッスンアドレス |
| `SHITTIM_CHEST_PORT` | `80` | リッスンポート |

### データベース

| 変数 | デフォルト | 目的 |
| --- | --- | --- |
| `SHITTIM_CHEST_DATABASE_URL` | `postgresql://sc:pass@localhost:5432/shittim_chest` | PostgreSQL接続文字列 |
| `SHITTIM_CHEST_DATABASE_MAX_CONNECTIONS` | `10` | SeaORM接続プールサイズ |

データベースとユーザーを作成:

```sql
CREATE USER sc WITH PASSWORD 'pass';
CREATE DATABASE shittim_chest OWNER sc;
```

### JWTと暗号化

| 変数 | デフォルト | 目的 |
| --- | --- | --- |
| `JWT_SECRET` | `change-me-in-production` | scepterと共有する秘密鍵。**一致させる必要あり** |
| `JWT_EXPIRATION_SECONDS` | `3600` | アクセストークン有効期間（1時間） |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | リフレッシュトークン有効期間（7日間） |
| `SHITTIM_CHEST_ENCRYPTION_KEY` | `change-me-32-bytes-base64-encoded` | APIキーとOAuthトークン用AES-256-GCMキー |

プロダクションキーを生成:

```bash
openssl rand -base64 32
```

### LLMプロバイダー（スタンドアロン操作用）

entelecheiaなしでshittim-chestを独立して使用するためにこれらを設定します:

| 変数 | 目的 |
| --- | --- |
| `LLM_DEFAULT_PROVIDER_ENDPOINT` | OpenAI互換APIエンドポイント（例: `https://api.deepseek.com/v1`） |
| `LLM_DEFAULT_PROVIDER_API_KEY` | プロバイダーのAPIキー |
| `LLM_DEFAULT_PROVIDER_MODELS` | カンマ区切りモデルリスト（例: `deepseek-chat,deepseek-reasoner`） |
| `LLM_DEFAULT_PROVIDER_CATEGORY` | プロバイダーカテゴリ: `chat`または`image` |
| `LLM_STREAM_BUFFER_SECONDS` | ストリームバッファタイムアウト（デフォルト: 60） |
| `LLM_MAX_TOKENS_DEFAULT` | デフォルト最大トークン数（デフォルト: 4096） |
| `LLM_REQUEST_TIMEOUT_SECONDS` | HTTPリクエストタイムアウト（デフォルト: 120） |

### リモートデバイス

| 変数 | デフォルト | 目的 |
| --- | --- | --- |
| `REMOTE_DEVICES_ENABLED` | `false` | リモートデバイス機能を有効化 |
| `REMOTE_DEVICES_SCEPTER_SOCK` | `/run/entelecheia/device_stream.sock` | デバイスデータ用Unixソケット |
| `REMOTE_DEVICES_FRAME_BUFFER_SIZE` | `4194304` | フレームバッファサイズ（バイト単位） |
| `REMOTE_DEVICES_MAX_SESSIONS_PER_USER` | `3` | ユーザーあたりの最大同時デバイスセッション数 |
| `WEBRTC_ICE_SERVERS` | `stun:stun.l.google.com:19302` | ICEサーバーリスト |

### GitHub OAuth

| 変数 | 目的 |
| --- | --- |
| `GITHUB_CLIENT_ID` | GitHub OAuth AppクライアントID |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth Appクライアントシークレット |
| `GITHUB_REDIRECT_URI` | OAuthコールバックURL（例: `https://your-domain/api/auth/github/callback`） |

### Scepter接続（プロキシ機能用）

| 変数 | デフォルト | 目的 |
| --- | --- | --- |
| `ENTELECHEIA_SCEPTER_URL` | `http://localhost:8424` | scepterのHTTPエンドポイント |
| `ENTELECHEIA_SCEPTER_WS_URL` | `ws://localhost:8424` | scepterのWebSocketエンドポイント |
| `ENTELECHEIA_TUI_SOCK` | `/run/entelecheia/entelecheia.sock` | トリガー転送用Unixソケット |

### Webhook

| 変数 | 目的 |
| --- | --- |
| `WEBHOOK_GITHUB_SECRET` | GitHub webhook検証用HMAC秘密鍵 |
| `WEBHOOK_GITLAB_SECRET` | GitLab webhook検証用トークン |
| `WEBHOOK_PUBLIC_URL` | webhookエンドポイントの公開URL |

## データベースセットアップ

```bash
just db-init      # スキーマ作成（SeaORMマイグレーション実行）
just db-migrate   # 保留中のマイグレーションを適用
```

### スキーマ概要

shittim_chest_dbはユーザー向けデータを所有します:

| テーブル | 目的 |
| --- | --- |
| `auth_users` | argon2パスワードハッシュ付きユーザーアカウント |
| `sessions` | リフレッシュトークン付きアクティブセッション |
| `api_keys` | APIキーレコード（ハッシュ化） |
| `oauth_connections` | サードパーティOAuthバインディング（GitHub） |
| `conversations` | チャット会話 |
| `messages` | ツール呼び出しデータ付きチャットメッセージ |
| `llm_providers` | LLMプロバイダー設定（APIキー暗号化） |
| `remote_devices` | リモートデバイスレコード |
| `device_sessions` | アクティブデバイスセッション |
| `channel_configs` | マルチプラットフォームチャネル設定 |
| `channel_messages` | チャネルメッセージレコード |
| `channel_pairings` | チャネル対チャットペアリング |

データベースをリセット:

```bash
just db-reset
```

## バックエンド開発

```bash
just dev-backend
```

これは`cargo run --package shittim_chest`を実行します。サーバーは`:80`で起動します。

### CLIコマンド

```bash
shittim_chest db-init      # データベーススキーマ作成
shittim_chest db-migrate   # 保留中のマイグレーションを適用
shittim_chest db-reset     # スキーマを削除して再作成
shittim_chest server       # Webサーバー起動
```

### ホットリロード

```bash
cargo install cargo-watch
cargo watch -x 'run --package shittim_chest -- server'
```

### APIエンドポイント概要

| ルートグループ | 目的 |
| --- | --- |
| `/api/auth/*` | ログイン、登録、GitHub OAuth、リフレッシュ、ログアウト |
| `/api/chat/*` | 会話、メッセージ、SSE/WSストリーミング、検索、エクスポート |
| `/api/providers/*` | LLMプロバイダーCRUD、APIキー管理、テスト |
| `/api/generation/*` | 画像生成、モデル一覧 |
| `/api/devices/*` | リモートデバイス一覧、セッション、WebRTCシグナリング |
| `/api/webhook/*` | GitHub/GitLab/Gitee/カスタムwebhookイングレス |
| `/api/proxy/*` | scepterへのリバースプロキシ（HTTP + WebSocket） |
| `/static/*` | SPA静的ファイルホスティング |

## フロントエンド開発

### 依存関係のインストール

```bash
pnpm install
```

### webui

```bash
just dev    # フロントエンドビルド + バックエンド起動 :3000
just watch  # ファイル変更時に自動再ビルド
```

両方のフロントエンドはViteによって`dist/`にビルドされます。バックエンドはこれらの静的ファイルを`:3000`で直接提供します — 別個のVite開発サーバーやプロキシは不要です。開発モードでは、`dev.py`がフロントエンドソースを監視し自動的に再ビルドします。

## クロスプロジェクトセットアップ

共有`arona`プロトコルクレートを使用したローカル開発では、ローカルのチェックアウトにパッチします。`~/.cargo/config.toml`を編集します（リポジトリにコミットしないでください）:

```toml
[patch.'https://github.com/celestia-island/arona']
arona = { path = "/path/to/arona" }
```

npmの場合、webuiは`arona`クレートのTSバインディングを`@celestia-island/arona`パスエイリアスを通じて消費し、`packages/webui/src/types/arona/`を指します。

## プロダクション向けビルド

```bash
just build
```

これは`cargo build --release`と`pnpm run build:all`を実行します。出力先:

- バックエンドバイナリ: `target/release/shittim_chest`
- フロントエンドアセット: `packages/webui/dist/`

### Docker

CLIラッパーを使用してビルドおよび実行（Docker APIを直接使用）:

```bash
just dev
```

または手動:

```bash
just build        # Dockerイメージビルド
just up           # 全サービス起動
just migrate      # データベースマイグレーション実行
```

プロダクションバイナリはAxumの静的ファイルミドルウェアを通じて`/`でフロントエンドアセットを提供します。別個のフロントエンドサーバーは不要です。

## 一般的な問題

### データベース接続拒否

```text
error: connection to server at "localhost", port 5432 failed
```

**修正**: PostgreSQLが実行中であり、`.env`の`SHITTIM_CHEST_DATABASE_URL`が設定と一致することを確認します。`psql "$SHITTIM_CHEST_DATABASE_URL" -c 'SELECT 1'`で検証してください。

### Scepterに到達不能

```text
error: error sending request for url (http://localhost:8424/...)
```

**修正**: entelecheia scepterインスタンスを起動するか、LLMプロバイダーを設定してスタンドアロンモードを使用します。バックエンドはチャット/画像生成においてscepterなしで動作します。

### ブラウザでCORSエラー

```text
Access-Control-Allow-Origin header is present on the requested resource
```

**修正**: 開発バックエンドは`localhost`オリジンに対してCORSを有効にします。ポートを変更した場合は、CORS設定を更新してください。プロダクションデプロイメントでは、CORSを処理するためにリバースプロキシ（nginx/caddy）を設定すべきです。

### pnpm installが失敗

**修正**: pnpm 9以上を使用していることを確認します。`corepack enable && corepack prepare pnpm@latest --activate`を実行して正しいバージョンをセットアップしてください。

### cargo buildが共有クレートで失敗

**修正**: `~/.cargo/config.toml`にローカルパッチがある場合、パスが存在し、クレート名が一致することを確認します。代わりにgit依存関係を使用するにはパッチセクションを削除してください。
