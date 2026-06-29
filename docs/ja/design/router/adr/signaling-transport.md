+++
title = "シグナリングトランスポート — デュアル Unix ソケット / TCP"
description = """アーキテクチャ意思決定記録 —— シグナリングトランスポート — デュアル Unix ソケット / TCP。"""
lang = "ja"
category = "design"
subcategory = "router"
+++

# シグナリングトランスポート — デュアル Unix ソケット / TCP

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

シグナリングクライアントは、WebRTC SDP オファー/アンサーおよび ICE 候補を交換するためにリレーサーバーに接続する。当初は Unix ドメインソケット（`tokio::net::UnixStream`）のみを使用していたが、これは Windows では利用できない。クロスプラットフォームサポートのためにフォールバックが必要である。

## Decision

デュアルトランスポートの `SignalingStream` 列挙型を実装し、まず Unix ドメインソケットを試行し（サポートされているプラットフォーム上で、パスが `/` で始まるか `.sock` で終わることで検出）、TCP にフォールバックする。`TransportWriter` アダプタは両方のトランスポートに対して `AsyncWrite` を抽象化する。非 Unix プラットフォームでは TCP のみが利用可能である。

## Consequences

### Positive

- クロスプラットフォームシグナリング。両方のトランスポート上で同一の JSON-RPC プロトコル

### Negative

- TCP シグナリングはデフォルトでは暗号化されない
- Windows ユーザーはループバックのみのバインディングを保証する必要がある
