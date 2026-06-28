+++
title = "組み込みフロントエンド戦略"
description = """shittim-chestは2つのフロントエンドホスティングモードをサポートします：開発モードでは、`dev.py`がフロントエンドソースを監視し、変更時に`pnpm build`をトリガーし、バックエンドが静的ファイルとA"""
lang = "ja"
category = "design"
subcategory = "webui"
+++

# 組み込みフロントエンド戦略

## 概要

shittim-chestは2つのフロントエンドホスティングモードをサポートします：開発モードでは、`dev.py`がフロントエンドソースを監視し、変更時に`pnpm build`をトリガーし、バックエンドが静的ファイルとAPIの両方を`:3000`で配信します；リリースモードでは、フロントエンドの静的ファイルがコンパイル時にRustバイナリに埋め込まれ、`:80`で配信されます。モードは`embedded-frontend` Cargoフィーチャーと`#[cfg(feature = "embedded-frontend")]`によるコードレベルの条件付きコンパイルで切り替えられます。

## アーキテクチャ比較

```mermaid
flowchart TB
    subgraph Dev[開発モード: dev.py + バックエンド]
        D1[dev.pyがフロントエンドsrcを監視] --> D2[pnpm build → dist/]
        D2 --> D3[shittim_chest :3000 が静的 + APIを配信]
    end
    subgraph Release[リリースモード: 組み込み]
        R1[ブラウザ] --> R2[shittim_chest :80]
        R2 --> R3[API + LLM]
        R2 --> R4[/static/*\n組み込みSPA]
    end
```

| 側面 | 開発（フィーチャーなし） | リリース（embedded-frontend） |
| --- | --- | --- |
| フロントエンドソース | Viteでビルド、バックエンドが配信 | `include_dir!`コンパイル時埋め込み |
| ホットリロード | dev.py経由で自動再ビルド | 非サポート（静的） |
| APIリクエストルーティング | ブラウザ直接接続（同一オリジン） | ブラウザ直接接続 |
| バイナリサイズ | バックエンドのみ | + フロントエンドdist/ディレクトリ |
| Node必要 | はい（ビルドのみ） | いいえ |
| 起動方法 | `dev.py`（監視 + 再ビルド） | `just up`ワンショット起動 |

## 実装詳細

### 条件付きコンパイル

```rust
# [cfg(feature = "embedded-frontend")]
static ARONA_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../dist/arona");

async fn serve_arona() -> impl IntoResponse {
    #[cfg(feature = "embedded-frontend")]
    {
        // コンパイル時に埋め込まれたDirから読み取り
    }
    #[cfg(not(feature = "embedded-frontend"))]
    {
        // ファイルシステムの./dist/arona/index.htmlから読み取り
    }
}
```

条件付きコンパイルは**関数本体レベル**で動作し、パブリックAPIは両方のモードで同一に保たれます。

### SPAフォールバック

アプリケーションはシングルページアプリケーションです。静的アセットにマッチしないすべてのルートは`index.html`を返します：

```text
GET /               → index.html
GET /chat/123       → index.html（フロントエンドルーターが処理）
GET /backend        → index.html
GET /backend/providers → index.html（フロントエンドルーターが処理）
```

### MIMEタイプ検出

静的ファイルの配信は、ファイル拡張子に基づいて正しいContent-Typeを返します：

| 拡張子 | Content-Type |
| --- | --- |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.html` | `text/html` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff/.woff2` | `font/woff2` |
| その他 | `application/octet-stream` |

## Dockerfileでのフロントエンドビルド

```text
ステージ1（フロントエンド）:
  node:22-slim → pnpm install → pnpm build:all → /app/dist/arona/

ステージ2（ビルダー）:
  rust:1.85-slim → COPY /app/dist/ → cargo build --features embedded-frontend

ステージ3（ランタイム）:
  debian:bookworm-slim → COPY バイナリ → ENTRYPOINT ["./shittim_chest"]
```

フロントエンドビルドとRustコンパイルは同じDockerfile内で完了します。最終的なランタイムイメージにはコンパイル済みバイナリのみが含まれます。

## 設計判断

1. **開発モードは自動再ビルドにdev.pyを使用**: `dev.py`がフロントエンドソースを監視し変更時に再ビルド、バックエンドが1つのポートですべてを配信します。
1. **リリースモードではリバースプロキシが不要**: バイナリがSPAを埋め込み、シングルプロセスデプロイメントを可能にし、運用の複雑さを軽減します。
1. **フロントエンドは実行時に動的ロードされない**: ファイルシステム依存とバージョンの不整合を回避します。リリースイメージには単一のバイナリファイルのみが含まれます。
1. **単一SPA**: フロントエンドは`/`で配信され、管理パネルは`/backend`にあります。
