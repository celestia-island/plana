+++
title = "Shittim Chest（什亭之匣）"
description = """[entelecheia](https://github.com/celestia-island/entelecheia) 多智能体平台的面向用户外壳"""
lang = "zhs"
category = "guides"
subcategory = "webui"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Shittim Chest logo" width="200"/>

# Shittim Chest（什亭之匣）

**[entelecheia](https://github.com/celestia-island/entelecheia) 多智能体平台的面向用户外壳**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fshittim--chest-blue.svg)](https://github.com/celestia-island/shittim-chest)

**[English](README.md)** &bull; **[简体中文](docs/guides/zhs/README.md)** &bull;
**[繁體中文](docs/guides/zht/README.md)** &bull; **[日本語](docs/guides/ja/README.md)** &bull;
**[한국어](docs/guides/ko/README.md)** &bull; **[Français](docs/guides/fr/README.md)** &bull;
**[Español](docs/guides/es/README.md)** &bull; **[Русский](docs/guides/ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **版本 0.1.0** — 活跃开发中。

[Entelecheia](https://github.com/celestia-island/entelecheia) 多智能体平台的 Web 界面、后端和 CLI。包括聊天、管理面板、认证、多频道集成和设备管理。

## 快速开始

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
just dev    # 后端在 :3000，前端在 :5173
```

**前置条件**：Rust 1.85+、Node 20+、pnpm 9+、[just](https://github.com/casey/just)、PostgreSQL 18+。

**[架构](ARCHITECTURE.md)** · **[贡献](CONTRIBUTING.md)** · **[安全](SECURITY.md)** · **[文档](docs/guides/en/)**

## 许可证

Business Source License 1.1 — 商业用途需要授权许可。非商业用途遵循 Synthetic Source License (SySL-1.0)；于 2030-01-01 完全转换为 SySL-1.0。
