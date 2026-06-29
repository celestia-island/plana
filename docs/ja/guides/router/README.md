+++
title = "Evernight"
description = """Evernight —— クロスプラットフォーム遠隔制御ライブラリとデーモン。"""
lang = "ja"
category = "guides"
subcategory = "router"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Evernight logo" width="200"/>

# Evernight

**あらゆるマシンに手が届く — クロスプラットフォームリモートコントロールライブラリとデーモン**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fevernight-blue.svg)](https://github.com/celestia-island/evernight)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — 初期開発段階。

クロスプラットフォームのリモートコントロール：画面キャプチャ、WebRTC ストリーミング、SSH リモートシェル、ファイル転送、ハードウェアテレメトリ、産業用プロトコル探査（Modbus、OPC-UA、MQTT）、TCP トンネリング、NAT トラバーサル。Rust ライブラリまたはスタンドアロンデーモンとして使用可能。

## クイックスタート

```bash
cargo build --release

# ハードウェア一覧
./target/release/evernight --list-hardware

# リモート SSH コマンド
./target/release/evernight --remote-exec "uname -a" --shell-host 192.168.1.100 --shell-user root --shell-key ~/.ssh/id_ed25519

# TCP トンネル
./target/release/evernight --forward-tunnel 8080:internal.example.com:80
```

**[はじめに](getting-started.md)** · **[ADR](../../design/en/)** · **[ドキュメント](./)**

## ライセンス

MIT
