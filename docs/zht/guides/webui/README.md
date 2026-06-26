+++
title = "Shittim Chest（什亭之匣）"
description = """[entelecheia](https://github.com/celestia-island/entelecheia) 多代理平台的使用者面向外殼"""
lang = "zht"
category = "guides"
subcategory = "webui"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Shittim Chest logo" width="200"/>

# Shittim Chest（什亭之匣）

**[entelecheia](https://github.com/celestia-island/entelecheia) 多代理平台的使用者面向外殼**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fshittim--chest-blue.svg)](https://github.com/celestia-island/shittim-chest)

**[English](README.md)** &bull; **[简体中文](docs/guides/zhs/README.md)** &bull;
**[繁體中文](docs/guides/zht/README.md)** &bull; **[日本語](docs/guides/ja/README.md)** &bull;
**[한국어](docs/guides/ko/README.md)** &bull; **[Français](docs/guides/fr/README.md)** &bull;
**[Español](docs/guides/es/README.md)** &bull; **[Русский](docs/guides/ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **版本 0.1.0** — 開發進行中。

[Entelecheia](https://github.com/celestia-island/entelecheia) 多代理平台的 Webui、後端和 CLI。包含聊天、管理面板、身分驗證、多頻道整合和裝置管理。

## 快速開始

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
just dev    # 後端於 :3000，前端於 :5173
```

**先決條件**：Rust 1.85+、Node 20+、pnpm 9+、[just](https://github.com/casey/just)、PostgreSQL 18+。

**[架構](ARCHITECTURE.md)** · **[貢獻](CONTRIBUTING.md)** · **[安全性](SECURITY.md)** · **[文件](docs/guides/en/)**

## 授權

Business Source License 1.1 — 商業用途需要授權。非商業用途依據 Synthetic Source License (SySL-1.0)；於 2030-01-01 完全轉換為 SySL-1.0。
