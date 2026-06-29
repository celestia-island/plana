+++
title = "コンテナランタイムクライアント（Docker/Podman）"
description = """アーキテクチャ意思決定記録 —— コンテナランタイムクライアント（Docker/Podman）。"""
lang = "ja"
category = "design"
subcategory = "router"
+++

# コンテナランタイムクライアント（Docker/Podman）

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

コンテナ（Docker、Podman）の管理は、ユニバーサル接続マネージャにとって主要なユースケースである。操作には、コンテナの一覧表示、exec シェル、ログ表示、ポート転送が含まれる。

## Decision

Unix ソケット（Windows では名前付きパイプ）を介して Docker Engine API を使用する。Podman の Docker 互換 API は同一のコードパスでサポートされる。型付きコンテナモデル（`ContainerInfo`、`ContainerState`、`ContainerPort`）を定義する。将来: Docker exec attach を介した対話型コンテナシェルのために `TerminalBackend` を実装する。

## Consequences

### Positive

- Docker API は十分に文書化されており安定している。Podman 互換性は無償で得られる

### Negative

- Docker/Podman デーモンが実行中である必要がある
- Docker バージョン間での API バージョンの差異
