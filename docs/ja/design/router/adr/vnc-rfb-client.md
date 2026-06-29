+++
title = "VNC (RFB) プロトコルクライアント"
description = """アーキテクチャ意思決定記録 —— VNC (RFB) プロトコルクライアント。"""
lang = "ja"
category = "design"
subcategory = "router"
+++

# VNC (RFB) プロトコルクライアント

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight は VNC をサポートするグラフィカルリモートデスクトップクライアントを必要とする。RFB プロトコル（RFC 6143）は VNC の標準である。選択肢: 既存の Rust クレートを使用する、libvncclient にバインディングする、またはスクラッチから実装する。

## Decision

スクラッチからピュア Rust RFB 003.008 クライアントを実装する。バージョンハンドシェイク、セキュリティネゴシエーション（None、VncAuth）、DES チャレンジレスポンス認証、ピクセルフォーマットネゴシエーション、Raw/CopyRect エンコーディングをサポートする。フロントエンド統合のために `ViewportBackend` トレイトを実装する。

## Consequences

### Positive

- C 依存なし。プロトコルフローを完全に制御可能

### Negative

- ZRLE および Tight エンコーディングはまだ実装されていない（実装が重い）
- VNC 認証に DES 実装が必要
