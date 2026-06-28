+++
title = "ビルドガイド"
description = """- [前提条件](#前提条件)"""
lang = "ja"
category = "guides"
subcategory = "core"
+++

# ビルドガイド

---

## 目次

- [前提条件](#前提条件)
- [インストール](#インストール)
- [設定](#設定)
- [ビルド](#ビルド)
- [実行](#実行)
- [データベース管理](#データベース管理)
- [開発環境](#開発環境)
- [デプロイ](#デプロイ)
- [トラブルシューティング](#トラブルシューティング)
- [Webhook ボットの実行](#webhook-ボットの実行)

---

## 前提条件

### システム要件

- **OS**: Linux、macOS または Windows（Docker CLI が必要）
- **メモリ**: 最低 8GB、推奨 16GB
- **ストレージ**: 最低 20GB の空き容量
- **CPU**: 4 コア以上推奨

> 説明（設計意図）
> Windows 側のコア要件は Docker CLI が利用可能であることです。コマンドは PowerShell または Windows Terminal で直接実行できます。
> ただし、コンテナは最終的に Linux ランタイムを必要とします：
> 1. ローカルソリューションは通常 Docker Desktop（一般的に WSL2 バックエンドに依存）。
> 2. 代替案として、ホストに Docker CLI のみをインストールし、`docker context` でリモート Linux Docker ホストに転送します。

### ソフトウェア要件

#### 必須ソフトウェア

- **Docker または Podman**（コンテナランタイム環境）

```bash
docker --version
docker compose version
```

現在のプラットフォームに応じて公式推奨のインストール方法を使用してください：

- Linux：Docker Engine、Docker Desktop for Linux、またはディストリビューション付属の Podman をインストール
- macOS：Docker Desktop または Podman Desktop をインストール
- Windows：Docker Desktop または Podman Desktop をインストール

**重要事項**：

- PostgreSQL などのランタイム依存関係はコンテナ化環境に含まれています
- ただし、`just` レシピやリポジトリ内補助スクリプトを実行する場合、ホストには Python 3.8+ が必要です
- ホストに PostgreSQL を個別にインストールする必要はありません
- Windows ではコマンドを PowerShell または Windows Terminal で直接実行できますが、デプロイには利用可能な Docker/Podman Linux ランタイムが必要です。ローカルデプロイは通常、WSL2 バックエンド付きの Docker Desktop を使用することを意味します。ホストの Docker CLI/context でリモート Linux Docker ホストに転送することも可能です。

- **Rust 1.85+**（開発ビルドのみ必要）

```bash
rustup update stable
```

プラットフォームに応じて公式 rustup インストール方法を使用してください：

- Linux/macOS：<https://rustup.rs> にアクセス
- Windows：<https://rustup.rs> から `rustup-init.exe` をダウンロードして実行し、`rustup update stable` を実行

#### 推奨ソフトウェア

- **just**（コマンドランナー）

```bash
  # cargo を使用
  cargo install just

  # brew を使用（macOS）
  brew install just
  ```

- **VS Code** に rust-analyzer 拡張機能をインストール

---

## インストール

### ステップ 1: リポジトリのクローン

```bash
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia
```

### ステップ 2: 環境変数の設定

```bash
# .env.example から .env を作成した後、設定を編集
nano .env  # またはお好みのエディタを使用
```

現在のシェルまたはファイルマネージャーを使用して `.env.example` を `.env` にコピーしてください。

POSIX シェル：

```bash
cp .env.example .env
```

PowerShell：

```powershell
Copy-Item .env.example .env
```

#### 基本設定

```bash
# データベース設定（コンテナ内部で自動設定）
# DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
# DATABASE_MAX_CONNECTIONS=10

# LLM クイック初期化、起動後に ApoRia にインポート
# 単一 provider：
# LLM_API_KEY=your-api-key-here
# LLM_BASE_URL=https://api.openai.com/v1
# LLM_MODEL=gpt-4
# 複数 provider（セミコロン区切り）：
# LLM_API_KEY=key1;key2
# LLM_BASE_URL=https://api.one/v1;https://api.two/v1
# LLM_PROTOCOL=openai;openai,api-key
# LLM_MODEL_DEEP=model-a1,model-a2;model-b1
# LLM_MODEL_NORMAL=model-a3;model-b2
# LLM_MODEL_BASIC=model-a4;model-b3

# provider レベルのショートカットエントリ（推奨）
OPENAI_API_KEY=your-api-key-here
# ANTHROPIC_API_KEY=
# DEEPSEEK_API_KEY=
# DASHSCOPE_API_KEY=
# BIGMODEL_API_KEY=
# ZAI_API_KEY=

# WebSocket 設定
WS_BIND_ADDRESS=127.0.0.1:42470
WS_MAX_CONNECTIONS=100
```

#### LLM 環境変数設定の説明

> **重要**：現在の LLM provider 設定は ApoRia が一元管理します。環境変数は起動ブートストラップエントリとしてのみ機能し、長期的な設定ソースではなくなりました。

**動作メカニズム**：

1. TUI がサーバーを自動起動する必要がある場合、汎用 `LLM_*` クイック初期化変数、または `OPENAI_API_KEY` のような provider レベル変数を読み取ります。複数 provider 設定ではセミコロン区切りの並列配列を使用します：`LLM_API_KEY`、`LLM_BASE_URL`、`LLM_PROTOCOL`、`LLM_MODEL_DEEP`、`LLM_MODEL_NORMAL`、`LLM_MODEL_BASIC`。プログラミングパッケージ環境変数（`BIGMODEL_API_KEY_CODING_PRO` など）もセミコロン区切りで複数キーをサポートし、自動的に `(#2)`、`(#3)` と採番されます。カスタム provider は括弧内にドメイン名が表示されます。
1. サーバー起動前に、TUI はまず初期 provider 設定を `res/prompts/agents/aporia/config.toml` に事前書き込みします
1. 事前書き込み完了後は、ApoRia 設定と TUI の Models ページが基準となります
1. 既存で API キーが空でない provider は環境変数によって上書きされません

**推奨される使用方法**：

- 環境変数を使用して初回ブートストラップを完了
- 以降は Models ページまたは `res/prompts/agents/aporia/config.toml` で一元管理

### ステップ 3: サービスの起動

```bash
# Docker Compose で全サービスを起動
docker compose up -d

# または just コマンドを使用（インストール済みの場合）
just dev
```

---

## 設定

### LLM プロバイダー設定

Entelecheia（玄枢）は複数の LLM プロバイダーをサポートしています。優先するプロバイダーを設定してください：

#### OpenAI

```bash
OPENAI_API_KEY=sk-...
```

#### Anthropic

```bash
ANTHROPIC_API_KEY=sk-ant-...
```

#### ローカル LLM（Ollama）

```bash
# Models ページまたは res/prompts/agents/aporia/config.toml でローカル provider を設定
# endpoint = http://localhost:11434
# model = llama2
```

### Docker 設定

```bash
# Docker ソケット（通常自動検出）
DOCKER_HOST=unix:///var/run/docker.sock

# コンテナ設定
CONTAINER_NETWORK=entelecheia-network
CONTAINER_REGISTRY=127.0.0.1:5000
```

---

## ビルド

### 開発ビルド

```bash
# クイック開発ビルド
just build-dev
```

### 本番ビルド

```bash
# 最適化されたリリースビルド
just build
```

### 特定コンポーネントのビルド

```bash
# サーバーのみビルド
cargo build -p scepter

# TUI のみビルド
cargo build -p entelecheia-tui

# 特定のエージェントをビルド
cargo build -p haplotes
```

### ビルド成果物

ビルド完了後、以下が見つかります：

- **バイナリファイル**: `target/debug/` または `target/release/`
- **Docker イメージ**: `just dev` の実行中に自動ビルド

---

## 実行

### 開発モード

```bash
# 完全な開発環境を起動（TUI を含む）
just dev

# サーバーのみ起動（TUI なし）
just dev --no-tui

# クリーン起動（全データ削除）
just dev-clean
```

### 本番モード

```bash
# サーバーを起動
just server

# TUI クライアントを起動
just tui

# 全エージェントを起動
just agents-up
```

### 端末互換性パラメータ

TUI は ANSI エスケープシーケンス、マウスイベント、画像レンダリング（Sixel/Kitty プロトコル）に依存しています。SSH セッション、シリアルコンソール、CI ランナー、または古い端末エミュレータなどの制限された端末環境では、3 つの段階的デグレードパラメータを使用できます：

#### `--no-image-render`

すべての画像レンダリングを無効にします。その他の機能——色、マウス、差分リフレッシュ——は完全に正常に動作します。

```bash
just tui -- --no-image-render
```

適用シーン：端末が色とマウスをサポートしているが、Sixel/Kitty 画像プロトコルが欠けている場合（最も一般的なケース）。

#### `--no-ansi`

マウスキャプチャと特殊キーリスニングを無効にします。色と差分（部分）画面リフレッシュは保持されます。マウスイベントが端末の選択、コピーペースト、またはスクロールバック履歴に干渉する場合に便利です。

```bash
just tui -- --no-ansi
```

適用シーン：色は必要だが、マウスキャプチャが問題を引き起こす場合（端末マルチプレクサ、`screen`、基本 `tmux` 設定など）。

#### `--no-ansi-pure`

純粋な単色モード——最も積極的なデグレード。すべての ANSI 色を無効（グローバルに `Color::Reset` を強制）、マウスキャプチャを無効、フレームごとに全画面再描画。起動画面のロゴは純粋な ASCII アートバージョンに置き換えられます。このパラメータは `--no-ansi` を含意します。

```bash
just tui -- --no-ansi-pure
```

適用シーン：最小限の端末サポートで SSH、シリアルコンソール、`docker exec`、CI 環境を通じて実行する場合、または ANSI カラーコードを正しく処理できない任意の端末。

#### パラメータ比較

| 機能 | デフォルト | `--no-image-render` | `--no-ansi` | `--no-ansi-pure` |
| --- | --- | --- | --- | --- |
| 色 | 完全 | 完全 | 完全 | 無効 |
| マウスキャプチャ | はい | はい | いいえ | いいえ |
| 画像レンダリング | はい | いいえ | いいえ | いいえ |
| 画面リフレッシュ | 差分 | 差分 | 差分 | 全画面再描画 |
| 起動ロゴ | ANSI カラー | ANSI カラー | ANSI カラー | 純粋 ASCII アート |

### サービス管理

```bash
# サービス状態の確認
just dev-status

# ログの表示
just dev-logs

# サービスの停止
just dev-down

# 全サービスの強制終了
just dev-kill
```

---

## データベース管理

### データベースの初期化

```bash
# データベースの作成
just db-create

# マイグレーションの実行
just db-migrate

# シードデータで初期化
just db-init
```

### データベース操作

```bash
# データベース状態の確認
just db-status

# データベースのバックアップ
just db-backup

# データベースの復元
just db-restore backups/backup_xxx.sql

# データベースのリセット（警告：全データ削除）
just db-reset
```

### マイグレーション管理

```bash
# 新しいマイグレーションの作成
cargo test -p scepter test_create_migration -- --nocapture --ignored

# 直前のマイグレーションをロールバック
just db-migrate-down
```

---

## 開発環境

### 環境セットアップ

```bash
# すべての依存関係を初期化
just init

# Python 依存関係の確認

# コードのフォーマット
just fmt

# コードチェックの実行
just clippy
```

### テスト

```bash
# 全テストを実行
just test

# 特定タイプのテストを実行
just test unit
just test integration
just test e2e
just test llm-providers

# 詳細出力
just test verbose
```

### コード品質

```bash
# コードのフォーマット
just fmt

# フォーマットチェック
just fmt-check

# clippy の実行
just clippy

# 型チェック
just check
```

---

## デプロイ

### Docker デプロイ

#### イメージのビルド

```bash
docker build -t entelecheia:latest .
```

#### コンテナの実行

```bash
docker run -d --name entelecheia \
  --env-file .env \
  -p 8424:8424 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  entelecheia:latest
```

### Docker Compose デプロイ

```bash
# 全サービスを起動
docker compose up -d

# ログの表示
docker compose logs -f

# サービスの停止
docker compose down
```

---

## トラブルシューティング

### よくある問題

#### Docker 権限拒否

```bash
# ユーザーを docker グループに追加
sudo usermod -aG docker $USER

# ログアウトして再ログイン
```

#### ポートが既に使用中

```bash
# ポートを占有しているプロセスを確認
lsof -i :8424

# プロセスを終了
kill -9 <PID>
```

#### ビルド失敗

```bash
# ビルド成果物をクリーン
cargo clean

# 依存関係を更新
cargo update

# 再ビルド
just build
```

#### コンテナが起動できない

```bash
# Docker ログを確認
docker compose logs

# コンテナを再ビルド
docker compose down
docker compose build --no-cache
docker compose up -d
```

### ヘルプの取得

1. [GitHub Issues](https://github.com/celestia-island/entelecheia/issues) を検索
1. [ディスカッション](https://github.com/celestia-island/entelecheia/discussions) に参加

---

## Webhook ボットの実行

Webhook ボットは `plugins/github-webhook/` 配下にあります。各プラットフォームには独立したディレクトリがあります。

### 前提条件

- Python 3.10+（現在のボット）
- Node.js 18+（将来の TypeScript 移行）
- 各プラットフォームの bot トークン（[Webhook 設定ガイド](webhook-setup.md) を参照）

### 単一ボットの実行

```bash
# GitHub
cd plugins/github-webhook/github
pip install -r requirements.txt
python bot.py

# Gitee
cd plugins/github-webhook/gitee
pip install -r requirements.txt
python bot.py

# Discord
cd plugins/github-webhook/discord
pip install -r requirements.txt
python bot.py
```

### 全ボットの実行

```bash
just webhooks-up
```

### 環境変数

サンプル環境ファイルをコピーして設定：

```bash
cp plugins/github-webhook/.env.example plugins/github-webhook/.env
```

各プラットフォームの具体的な設定詳細は [Webhook 設定ガイド](webhook-setup.md) を参照してください。

---

## 次のステップ

- [基本ガイド](fundamentals.md) を読んでアーキテクチャを理解
- [エージェントドキュメント](../../agents/) を参照して利用可能なエージェントを確認

---

**ビルドをお楽しみください！** 🚀
