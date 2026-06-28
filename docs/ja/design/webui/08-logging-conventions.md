+++
title = "CLIログ規約"
description = """shittim-chest CLIラッパーのログ出力は、entelecheiaと一貫した規約に従い、`tracing`エコシステムを使用し、コンパクトで人間が読める形式でstderrに出力します。"""
lang = "ja"
category = "design"
subcategory = "webui"
+++

# CLIログ規約

## 概要

shittim-chest CLIラッパーのログ出力は、entelecheiaと一貫した規約に従い、`tracing`エコシステムを使用し、コンパクトで人間が読める形式でstderrに出力します。

## フレームワークの選択

| コンポーネント | 選択 | 理由 |
| --- | --- | --- |
| ログフレームワーク | `tracing` | Rustエコシステム標準、entelecheiaと一貫 |
| サブスクライバ | `tracing-subscriber` fmtレイヤー | コンパクト出力、JSONパース不要 |
| 時刻形式 | `ShortTimer`（HH:MM:SS） | ターミナルフレンドリー、entelecheia CLIと一貫 |
| 出力先 | stderr | stdoutから分離、パイプに干渉しない |

## 初期化コード

```rust
use chrono::Local;
use tracing_subscriber::fmt::time::FormatTime;

struct ShortTimer;

impl FormatTime for ShortTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{} ", now.format("%H:%M:%S"))
    }
}

// 初期化
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(&args.log_level))
    .with_target(false)          // モジュールパスを非表示
    .with_timer(ShortTimer)      // HH:MM:SS形式
    .compact()                   // コンパクトモード
    .with_writer(std::io::stderr) // stderrに出力
    .init();
```

## 形式比較

| モード | 出力例 | ユースケース |
| --- | --- | --- |
| CLI（現在） | `14:23:05  INFO ネットワーク shittim-chest を作成中...` | 開発、運用 |
| サーバー（将来） | `{"timestamp":"...","level":"INFO","message":"..."}` | 本番ログ収集 |

## --log-levelパラメータ

CLIは`--log-level` / `-l`パラメータを受け付けます（デフォルト`info`）：

```text
shittim-chest --log-level debug dev
shittim-chest -l trace status
```

サポートレベル: `trace`、`debug`、`info`、`warn`、`error`。

## ログレベルの使用規約

| レベル | 目的 | 典型的なCLIシナリオ |
| --- | --- | --- |
| `info` | 重要な操作 | コンテナ作成/起動/停止、マイグレーション開始/完了 |
| `warn` | 潜在的な問題 | マイグレーションリトライ、コンテナは存在するが異常状態 |
| `error` | エラー | コンテナクラッシュ、マイグレーション失敗、ネットワーク作成失敗 |
| `debug` | デバッグ情報 | （現在未使用、将来のために予約） |
| `trace` | 詳細フロー | （現在未使用、将来のために予約） |

## 設計原則

1. **CLIはエラーを飲み込まない**: すべてのエラーは`anyhow::Result`を介して上方に伝播し、`main()`が自動的にエラーチェーンを出力します。
1. **すべての操作開始にログがある**: `ネットワークを作成中...`、`マイグレーションを実行中...`、`shittim_chestをビルド中...` — ユーザーはCLIが何をしているかを把握できます。
1. **すべての操作完了に確認がある**: `shittim-chest が 0.0.0.0:80 で起動しました`、`すべてのサービスが起動しました`。
1. **成功した操作はログに残さない**: ノイズを避けるため、ネットワークが既に存在する場合`ensure_network`は出力しません。
1. **コンテナログはDocker API経由で取得**: CLI自体はビジネスログを書き込まず、オーケストレーション操作ログのみを出力します。

## entelecheiaとの整合性

| 機能 | entelecheia CLI | shittim-chest CLI | 整合 |
| --- | --- | --- | --- |
| フレームワーク | `tracing` | `tracing` | ✅ |
| 時刻形式 | `ShortTimer`（HH:MM:SS） | `ShortTimer`（HH:MM:SS） | ✅ |
| 出力先 | stderr | stderr | ✅ |
| コンパクトモード | `.compact()` | `.compact()` | ✅ |
| ターゲット非表示 | `.with_target(false)` | `.with_target(false)` | ✅ |
| --log-level | サポート | サポート | ✅ |

両プロジェクトのCLIログ出力は視覚的に同一であり、開発者が2つのプロジェクト間を容易に切り替えられます。
