+++
title = "Entelecheia"
description = """Rustベースのマルチエージェント協調プラットフォーム"""
lang = "ja"
category = "guides"
subcategory = "core"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Entelecheia logo" width="200"/>

# Entelecheia

**Rustベースのマルチエージェント協調プラットフォーム**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fentelecheia-blue.svg)](https://github.com/celestia-island/entelecheia)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **バージョン 0.2.0** — 初期開発段階。TUI が主要インターフェースです。WebUI は [shittim-chest](https://github.com/celestia-island/shittim-chest) にあります。

実行専用マイクロカーネルマルチエージェントプラットフォーム —— LLM は 3 つのツール（`exec`、`write_to_var`、`write_to_var_json`）のみ呼び出し可能。12 個の Layer1 エージェント、コンテナ隔離によるタスク実行、IEPL TypeScript パイプライン。

## クイックスタート

**Linux / macOS：**

```bash
curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
```

**Windows (WSL2)：**

```powershell
irm https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.ps1 | iex
```

**[アーキテクチャ](../../ARCHITECTURE.md)** · **[ビルド](building.md)** · **[セキュリティ](../../SECURITY.md)** · **[ドキュメント](./)**

## ライセンス

Business Source License 1.1 —— 商用利用にはライセンス許諾が必要です。非商用利用は SySL-1.0 プロトコルに従います。
