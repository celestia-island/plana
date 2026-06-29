+++
title = "非同期ランタイム — tokio"
description = """アーキテクチャ意思決定記録 —— 非同期ランタイム — tokio。"""
lang = "ja"
category = "design"
subcategory = "router"
+++

# 非同期ランタイム — tokio

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight は、ネットワーク I/O（SSH、WebRTC、TCP トンネル）、タイマーベースの操作（フレームキャプチャループ、プロトコルタイムアウト）、および並行タスク管理のために非同期ランタイムを必要とする。Rust エコシステムには `tokio`、`async-std`、`smol` が存在する。

## Decision

唯一の非同期ランタイムとして `tokio` を使用する。`tokio` は Rust 非同期エコシステムにおけるデファクトスタンダードであり、最も豊富なドライバサポートを備える。主要な依存クレート（`russh`、`webrtc`、`reqwest`）はすでに `tokio` を前提としている。同期コンテキストからの spawn には `tokio::runtime::Handle::current()` を使用する。

## Consequences

### Positive

- エコシステム互換性。ランタイムブリッジが不要

### Negative

- `tokio` は重い依存である
- `tokio` をサポートしない `async-std` や `smol` のクレートは使用できない
