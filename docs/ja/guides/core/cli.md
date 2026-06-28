+++
title = "CLI 使用ガイド"
description = """`entelecheia-cli` は Entelecheia（玄枢）マルチエージェント協調プラットフォームのコマンドラインインターフェースです。Unix ソケット JSON-RPC を通じて scepter サーバーと通信し、チャットインタラクション、サービスライフサイクル管理、エージェント制御、設定などの機能を提供します。"""
lang = "ja"
category = "guides"
subcategory = "core"
+++

# CLI 使用ガイド

`entelecheia-cli` は Entelecheia（玄枢）マルチエージェント協調プラットフォームのコマンドラインインターフェースです。Unix ソケット JSON-RPC を通じて scepter サーバーと通信し、チャットインタラクション、サービスライフサイクル管理、エージェント制御、設定などの機能を提供します。

> 説明：CLI は現在 TUI と完全に同等の機能に達していません。現在の状態については [ARCHITECTURE.md](../../ARCHITECTURE.md) を参照してください。

---

## 目次

- [インストール](#インストール)
- [基本的な使い方](#基本的な使い方)
- [グローバルオプション](#グローバルオプション)
- [チャットコマンド](#チャットコマンド)
- [エージェント管理](#エージェント管理)
- [サービスライフサイクル](#サービスライフサイクル)
- [設定](#設定)
- [接続コンテキスト](#接続コンテキスト)
- [状態と監視](#状態と監視)
- [サブスクリプション（Layer3）](#サブスクリプションlayer3)
- [エージェントの実行](#エージェントの実行)
- [タイムライン](#タイムライン)
- [Docker イメージ](#docker-イメージ)
- [高度な使い方](#高度な使い方)

---

## インストール

### ソースからのビルド

```bash
# リポジトリのクローン
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia

# CLI バイナリのビルド
cargo build --package entelecheia-cli

# または just を使用
just cli
```

バイナリファイルは `target/debug/entelecheia-cli`（debug）または `target/release/entelecheia-cli`（release）にあります。

### ビルド済みバイナリ

ビルド済みバイナリは [GitHub Releases](https://github.com/celestia-island/entelecheia/releases) から入手できます。お使いのプラットフォームに適したアーカイブをダウンロードし、バイナリを `PATH` に配置してください。

---

## 基本的な使い方

```bash
# ヘルプの表示
entelecheia-cli --help

# スキルチェーンを通じてメッセージを送信
entelecheia-cli send このプロジェクトのアーキテクチャを説明してください

# パイプでメッセージを送信
echo "このファイルを要約してください" | entelecheia-cli send

# システム状態の確認
entelecheia-cli status
```

---

## グローバルオプション

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | ログレベル（trace、debug、info、warn、error） | `warn` |
| `-d, --daemon` | バックグラウンドでコマンドをディスパッチ後すぐに終了 | — |
| `-c, --clean` | Cosmos コンテナとソケットファイルをクリーンアップ | — |
| `-a, --auto-approve` | 操作を自動承認（サーバーが実行中であることを確認） | — |
| `-t, --table` | 人間可読なテーブル出力（ANSI 形式） | デフォルト |
| `-j, --json` | JSON 出力（機械可読） | — |
| `-r, --raw` | 生のプレーンテキスト出力（フォーマットなし） | — |
| `--format <FORMAT>` | 出力形式（table、json、raw） | `table` |

出力形式オプション：

- `table` — 人間可読なテーブル出力
- `json` — 機械可読な JSON 出力

**例：**

```bash
# コンテナのクリーンアップ
entelecheia-cli --clean

# JSON 形式で状態を取得
entelecheia-cli status --format json

# デバッグモードでメッセージを送信
entelecheia-cli -l debug send "接続問題のデバッグ"

# バックグラウンドモードでエージェントを実行（即時復帰）
entelecheia-cli -d run my-agent --ci
```

---

## チャットコマンド

`chat` サブコマンドはセッションエージェントシステムとの対話を管理します。

### メッセージの送信

```bash
entelecheia-cli chat send [OPTIONS]
```

| オプション | 説明 |
| --- | --- |
| `-m, --message <MSG>` | 送信するメッセージテキスト |
| `--stdin` | 標準入力からメッセージを読み取り |
| `-f, --file <PATH>` | ファイルからメッセージを読み取り |

一度に 1 つの入力ソースのみ使用できます。

**例：**

```bash
# メッセージを直接送信
entelecheia-cli chat send -m "こんにちは、何ができますか？"

# 標準入力から
echo "src/main.rs のコードを分析してください" | entelecheia-cli chat send --stdin

# ファイルから
entelecheia-cli chat send -f ./prompts/review.txt
```

`chat send` コマンドはメッセージを**スキルチェーン**——複数のエージェントを協調させるコア実行パイプライン——を通じて送信します。実行中はスピナーアニメーションで進捗が表示されます。

### チャット履歴

```bash
entelecheia-cli chat history [OPTIONS]
```

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `--conversation <ID>` | 会話 ID でフィルタ | — |
| `--agent <TYPE>` | エージェントタイプでフィルタ | — |
| `--role <ROLE>` | ロールでフィルタ（user/assistant/system） | — |
| `--from <ISO8601>` | 開始日時（ISO 8601） | — |
| `--to <ISO8601>` | 終了日時（ISO 8601） | — |
| `--limit <N>` | 返す最大メッセージ数 | `50` |
| `--offset <N>` | ページネーションオフセット | `0` |

**例：**

```bash
entelecheia-cli chat history --agent ApoRia --limit 20 --from 2026-05-01T00:00:00Z
```

### 最近のメッセージ

```bash
entelecheia-cli chat recent [OPTIONS]
```

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `--timeline <ID>` | タイムライン/会話 ID でフィルタ | — |
| `--agent <TYPE>` | エージェントタイプでフィルタ | — |
| `--limit <N>` | 返す最大メッセージ数 | `20` |

---

## エージェント管理

エージェントのライフサイクルを管理（一覧表示、起動、停止、再起動）。

```bash
entelecheia-cli agent <COMMAND>
```

### コマンド

```bash
# 全エージェントとその状態を一覧表示
entelecheia-cli agent list

# タイプでエージェントを起動
entelecheia-cli agent start <AGENT_TYPE>

# 実行中のエージェントを停止
entelecheia-cli agent stop <AGENT_TYPE>

# エージェントを再起動
entelecheia-cli agent restart <AGENT_TYPE>
```

**利用可能なエージェントタイプ：** ApoRia、EleOs、EpieiKeia、Haplotes、HubRis、Kalos、NeiKos、OreXis、PhiLia、Polemos、SkeMma、SkoPeo。

> 説明：エージェントは scepter ランタイム内でライブラリ crate として実行され、独立した実行可能ファイルではありません。`agent start` コマンドはエージェント名に一致するバイナリを生成しようとしますが、これは主にエージェントが個別のバイナリとしてコンパイルされている場合に適用されます。実際の使用では、エージェントは scepter サーバーを通じてアクティブ化されます。

---

## サービスライフサイクル

Docker コンテナを使用して Entelecheia（玄枢）サービススタックを管理します。

### サービスの初期化

```bash
entelecheia-cli init [OPTIONS]
```

完全なサービススタックをセットアップ：PostgreSQL（pgvector 付き）、Docker レジストリ、scepter サーバー、WebUI。必要な Docker ネットワークを作成し、イメージをプル/ビルドします。

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `--prefix <STR>` | コンテナ名プレフィックス | `e-` |
| `--source-build` | プルではなくソースからイメージをビルド | `false` |
| `--webui-port <PORT>` | WebUI ポート | `3424` |

**例：**

```bash
entelecheia-cli init --prefix ent- --webui-port 8080
```

### 全サービスの起動

```bash
entelecheia-cli serve [OPTIONS]
```

以前に初期化されたすべてのコンテナを起動します。事前に `init` を実行する必要があります。

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `--prefix <STR>` | コンテナ名プレフィックス | `e-` |
| `--webui-port <PORT>` | WebUI ポート | `3424` |

### 全サービスの停止

```bash
entelecheia-cli stop [OPTIONS]
```

実行中のすべてのコンテナを順番に停止：webui → scepter → registry → postgres。

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `--prefix <STR>` | コンテナ名プレフィックス | `e-` |

### WebUI のみ起動

```bash
entelecheia-cli webui [OPTIONS]
```

WebUI コンテナのみを起動または作成します。

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `--prefix <STR>` | コンテナ名プレフィックス | `e-` |
| `--webui-port <PORT>` | WebUI ポート | `3424` |

---

## 設定

システム設定の表示と検証。

### 設定の表示

```bash
entelecheia-cli config show
```

現在の設定を表示：

- データベース URL と接続設定
- ApoRia LLM プロバイダー設定（名前、モデル、エンドポイント）
- WebSocket バインドアドレス
- ログレベル

API キーは出力でマスキングされます（`***` と表示）。

### 設定の検証

```bash
entelecheia-cli config validate
```

検証チェックを実行：

- データベース URL が設定されている
- 少なくとも 1 つの ApoRia プロバイダーが完全な設定で構成されている
- WebSocket バインドアドレスが設定されている

問題の詳細とともに合格/不合格の結果を返します。

**出力例：**

```text
Validate Configuration:

Validating database configuration...
  [ OK ]  Database URL set

Validating ApoRia LLM configuration...
  [ OK ]  ApoRia providers configured

Validating WebSocket configuration...
  [ OK ]  WebSocket Bind Address set

[ OK ]  Configuration validation passed
```

---

## 接続コンテキスト

`context` サブコマンドは名前付き接続プロファイルを管理し、ローカル（Unix ソケット）とリモート（WebSocket）scepter サーバー間の切り替えを可能にします。使用方法は Docker の `docker context` コマンドに似ています。

### 概念

**コンテキスト**は、CLI が scepter サーバーに接続する方法を記録する名前付き設定プロファイルです：

- **local** — Unix ソケット接続（デフォルト、自動的に `/run/.../entelecheia-tui.sock` に解決）
- **remote** — Bearer トークン認証付き WebSocket 接続

コンテキストは `~/.config/entelecheia/contexts/contexts.toml` に保存されます。

### コンテキストの一覧表示

```bash
entelecheia-cli context list
```

現在アクティブなコンテキストは `*` でマークされます。

### 現在のコンテキストの表示

```bash
entelecheia-cli context show
```

アクティブなコンテキストのタイプ、ソケットパス、WS URL、説明情報を表示します。

### コンテキストの作成

```bash
# リモート WebSocket コンテキスト
entelecheia-cli context create staging \
  --ws-url ws://scepter.example.com:8424/ws \
  --bearer-token <TOKEN> \
  --description "Staging server"

# 追加のローカルコンテキスト
entelecheia-cli context create dev --description "Development server"
```

リモートサーバーから Bearer トークンを取得：

```bash
# サーバーマシン上で
docker exec e-scepter cat /home/entelecheia/.config/entelecheia/scepter.token
```

### コンテキストの切り替え

```bash
entelecheia-cli context use staging
# 以降、すべてのコマンド（send、status、chat など）は staging 接続を通じてルーティングされます
```

### コンテキストの削除

```bash
entelecheia-cli context remove staging
```

`default` コンテキストは削除できません。

### ワークフロー例

```bash
# 現在のコンテキストを表示
entelecheia-cli context list

# ステージングサーバー用のリモートコンテキストを作成
entelecheia-cli context create staging \
  --ws-url ws://192.168.1.100:8424/ws \
  --bearer-token $(cat /path/to/token)

# ステージング環境に切り替え
entelecheia-cli context use staging

# リモートサーバーを通じてメッセージを送信
entelecheia-cli send "現在の TODO を一覧表示"

# リモートサーバーの状態を確認
entelecheia-cli status

# ローカルに切り替え戻し
entelecheia-cli context use default
```

---

## 状態と監視

### システム状態

```bash
entelecheia-cli status
```

表示内容：

- サーバーバージョン
- 接続状態（ソケット状態）
- LLM プロバイダー概要
- WebSocket バインドアドレス
- エージェント一覧と実行/停止状態
- システムリソース（メモリ使用量、平均負荷）

### 状態パスクエリ

`status` コマンドは、特定のサブシステムを照会するためのパス形式パラメータを受け付けます。構文はエージェント範囲のタイムライン、チャット履歴チェック、デバイス列挙をサポートします。

```bash
entelecheia-cli status <PATH> [--raw]
```

| パス構文 | 説明 |
| --- | --- |
| `timeline.#agent[-N]` | あるエージェントの最近 N 回の skill 呼び出し記録を表示 |
| `timeline.#agent[N][M]` | N 回目の skill 呼び出しにおける M 番目の MCP/ツール呼び出しを表示 |
| `history[-N]` | 最近 N 件のチャットメッセージを表示（全ロール） |
| `history[-N].body` | 最後から N 番目のメッセージの本文を表示 |
| `device` | すべての Polemos が認識するエッジデバイスを一覧表示 |
| `device[N]` | N 番目の Polemos デバイスの詳細情報を表示 |

**例：**

```bash
# Haplotes #001 エージェントの最近 30 回の skill スケジューリング履歴
entelecheia-cli status timeline.#hap_lotes.001[-30]

# 3 回目の skill 呼び出しの 2 番目の MCP/ツール呼び出し
entelecheia-cli status timeline.#hap_lotes.001[3][2]

# 最近 30 件のメッセージ
entelecheia-cli status history[-30]

# 最後から 3 番目のメッセージ本文（プレーンテキスト）
entelecheia-cli status history[-3].body --raw

# すべての Polemos デバイス
entelecheia-cli status device

# 3 番目の Polemos デバイス詳細
entelecheia-cli status device[3]
```

> **シェル注意:** bash/zsh では、glob 展開を防ぐために `[...]` を含むパスをシングルクォートで囲んでください：`entelecheia-cli status 'history[-30]'`。`#` 文字が単語の中間に埋め込まれている場合はエスケープ不要です。fish シェルでは、上記パスはいずれもクォート不要です。

状態パスクエリは Unix ソケット JSON-RPC を通じてサーバーと通信します。`timeline.*` と `history.*` クエリはサーバーが実行中である必要があります。`device` クエリはサーバー上に Polemos ワークスペース登録が必要です。

### ログの表示

```bash
entelecheia-cli logs [OPTIONS]
```

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `-a, --agent <NAME>` | エージェント名でログをフィルタ | 全エージェント |
| `-l, --lines <N>` | 表示する行数（末尾） | `100` |

**例：**

```bash
# 全エージェントログの最後の 200 行を表示
entelecheia-cli logs -l 200

# ApoRia ログを表示
entelecheia-cli logs -a ApoRia
```

ログは `./logs/` ディレクトリから読み取られます。各エージェントにはそれぞれのログファイル（`ApoRia.log`、`EleOs.log` など）があります。

---

## サブスクリプション（Layer3）

Layer3 エージェントサブスクリプションを管理——インストールおよび実行可能な外部エージェントパッケージ。

### サブスクリプションの一覧表示

```bash
entelecheia-cli subscribe list
```

設定されているすべてのサブスクリプションを表示（状態：インストール済み/保留中、有効状態、自動更新設定、ソース）。

### サブスクリプションの追加

```bash
entelecheia-cli subscribe add [OPTIONS]
```

| オプション | 説明 |
| --- | --- |
| `--name <NAME>` | サブスクリプション名（必須） |
| `--source <SOURCE>` | ソースタイプ：`official`、`github` または `url`（必須） |
| `--repository <REPO>` | GitHub リポジトリ（github ソース用） |
| `--url <URL>` | 直接 URL（url ソース用） |
| `--version <VER>` | バージョン制約 |
| `--auto-update` | 自動更新を有効化 |
| `--disabled` | 無効状態で追加 |

**例：**

```bash
entelecheia-cli subscribe add --name my-agent --source github --repository user/repo
```

### サブスクリプションの削除

```bash
entelecheia-cli subscribe remove <NAME>
```

### サブスクリプションの同期

```bash
# 全サブスクリプションを同期
entelecheia-cli subscribe sync

# 特定のサブスクリプションを同期
entelecheia-cli subscribe sync --name my-agent
```

### 自動更新

```bash
entelecheia-cli subscribe auto-update
```

`auto_update` が有効なすべてのサブスクリプションを更新します。

---

## エージェントの実行

```bash
entelecheia-cli run <AGENT> [OPTIONS]
```

Layer3 エージェントスクリプトを実行します。現在のディレクトリで `.amphoreus/<AGENT>/run.py` を検索します。初回実行時に事前チェック監査が実行されます。

| オプション | 説明 |
| --- | --- |
| `--ci` | CI モードを有効化 |
| `--auto-pr` | 自動 PR モードを有効化 |
| `--dry-run` | ドライラン（実際の変更を行わない） |
| `--providers <LIST>` | カンマ区切りのプロバイダーリスト |
| `--output-dir <DIR>` | 出力ディレクトリ |

**例：**

```bash
# ドライランモードで Layer3 エージェントを実行
entelecheia-cli run my-agent --dry-run

# 指定されたプロバイダーで実行
entelecheia-cli run my-agent --providers openai,anthropic

# CI モードで PR を自動提出
entelecheia-cli run my-agent --ci --auto-pr

# バックグラウンドモードで実行（即時復帰、子プロセスがバックグラウンド実行）
entelecheia-cli -d run my-agent --ci --auto-pr
```

### バックグラウンドモード（`-d` / `--daemon`）

バックグラウンドモードフラグは、CLI が `--daemon` パラメータを除去した形で分離された子プロセスを再生成し、即座に復帰します。子プロセスは元のコマンドを継承し、独立して実行されます。その後 `status` で進捗を確認できます。

`run`、`init`、`deploy` などの長時間実行操作に適しています：

```bash
# エージェント実行をバックグラウンドでディスパッチ
entelecheia-cli -d run my-agent

# サービス初期化をバックグラウンドでディスパッチ
entelecheia-cli -d init --prefix prod-

# 後で状態を確認
entelecheia-cli status
entelecheia-cli status history[-5]
```

---

## タイムライン

会話タイムラインを表示します。

### タイムラインの一覧表示

```bash
entelecheia-cli timeline list [OPTIONS]
```

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `--agent <TYPE>` | エージェントタイプでフィルタ | — |
| `--limit <N>` | 最大結果数 | `50` |
| `--offset <N>` | ページネーションオフセット | `0` |

### タイムライン詳細の表示

```bash
entelecheia-cli timeline show <CONVERSATION_ID> [OPTIONS]
```

| オプション | 説明 | デフォルト値 |
| --- | --- | --- |
| `--include-messages` | 出力にメッセージを含める | `true` |

---

## Docker イメージ

```bash
entelecheia-cli init-docker-images [OPTIONS]
```

プラットフォームに必要な Docker イメージをビルドまたはプルします。

| オプション | 説明 |
| --- | --- |
| `--source-build` | プルではなくソースからイメージをビルド |
| `--tag <TAG>` | イメージタグ（デフォルト：`latest`） |

**例：**

```bash
# ソースから全イメージをビルド
entelecheia-cli init-docker-images --source-build

# カスタムタグでプル
entelecheia-cli init-docker-images --tag v0.2.0
```

管理対象イメージ：

- `entelecheia` — オーケストレーションサーバー（組み込み cosmos ランタイム付き）
- `pgvector/pgvector` — ベクトル拡張付き PostgreSQL

---

## 高度な使い方

### スクリプト用 JSON 出力

`--format json` を使用して機械可読な出力を取得し、`jq` や他のツールにパイプできます：

```bash
entelecheia-cli status --format json | jq '.server_version'
entelecheia-cli chat history --format json | jq '.messages[].content'
```

### クリーンアップと初期化のチェーン

```bash
# 完全な解体と再構築
entelecheia-cli --clean && entelecheia-cli init --prefix my-
```

### デバッグモード

```bash
# トレースレベルログを有効にしてデバッグ
entelecheia-cli -l trace send "テストメッセージ"
```

### TUI との併用

CLI と TUI は同じ scepter サーバーに接続します。両方を同時に使用できます：

- TUI を起動してインタラクティブセッション：`cargo run --bin entelecheia-tui`
- CLI を使用してスクリプト作成、自動化、クイッククエリ

---

## トラブルシューティング

### "No command specified"

`--help` を実行して利用可能なコマンドを確認するか、`send "メッセージ"` でクイックにメッセージを送信します。

### "Failed to connect to Docker"

Docker（または Podman）が実行中であることを確認：

```bash
docker info
docker run hello-world
```

### "Agent binary not found"

エージェントは scepter ランタイムの内部ライブラリ crate であり、独立したバイナリではありません。scepter サーバーを起動してエージェントをアクティブ化します：

```bash
entelecheia-cli init && entelecheia-cli serve
```

### "No LLM providers configured"

環境変数を通じて ApoRia プロバイダー設定を行います。プロバイダー設定の説明については [ビルドガイド](building.md) を参照してください。

### "Configuration validation failed"

`entelecheia-cli config validate` を実行して、どのチェックが失敗したかを確認します。よくある問題：

- `DATABASE_URL` 環境変数が不足
- ApoRia プロバイダー設定が不完全（名前、モデル、`api_key`）
- WebSocket バインドアドレスが不足
