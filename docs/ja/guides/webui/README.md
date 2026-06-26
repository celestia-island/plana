+++
title = "Shittim Chest（什亭之匣）"
description = """[entelecheia](https://github.com/celestia-island/entelecheia) マルチエージェントプラットフォームのユーザー向けシェル"""
lang = "ja"
category = "guides"
subcategory = "webui"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Shittim Chest ロゴ" width="200"/>

# Shittim Chest（什亭之匣）

**[entelecheia](https://github.com/celestia-island/entelecheia) マルチエージェントプラットフォームのユーザー向けシェル**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fshittim--chest-blue.svg)](https://github.com/celestia-island/shittim-chest)

**[English](README.md)** &bull; **[简体中文](docs/guides/zhs/README.md)** &bull;
**[繁體中文](docs/guides/zht/README.md)** &bull; **[日本語](docs/guides/ja/README.md)** &bull;
**[한국어](docs/guides/ko/README.md)** &bull; **[Français](docs/guides/fr/README.md)** &bull;
**[Español](docs/guides/es/README.md)** &bull; **[Русский](docs/guides/ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **バージョン 0.1.0** — 活発に開発中。

[Entelecheia](https://github.com/celestia-island/entelecheia)マルチエージェントプラットフォーム向けのWebui、バックエンド、CLI。チャット、管理パネル、認証、マルチチャネル統合、デバイス管理を含みます。

## クイックスタート

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
just dev    # バックエンド :3000、フロントエンド :5173
```

**前提条件**: Rust 1.85以上、Node 20以上、pnpm 9以上、[just](https://github.com/casey/just)、PostgreSQL 18以上。

**[アーキテクチャ](ARCHITECTURE.md)** · **[貢献](CONTRIBUTING.md)** · **[セキュリティ](SECURITY.md)** · **[ドキュメント](docs/guides/en/)**

## ライセンス

Business Source License 1.1 — 商用利用にはライセンスが必要です。非商用利用はSynthetic Source License (SySL-1.0)の下で許可され、2030年1月1日に完全にSySL-1.0に移行します。
