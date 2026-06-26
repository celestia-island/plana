+++
title = "Entelecheia"
description = """基於 Rust 的多智慧體協作平台"""
lang = "zht"
category = "guides"
subcategory = "core"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Entelecheia logo" width="200"/>

# Entelecheia

**基於 Rust 的多智慧體協作平台**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fentelecheia-blue.svg)](https://github.com/celestia-island/entelecheia)

**[English](../../README.md)** &bull; **[簡體中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **版本 0.2.0** — 早期開發階段。TUI 是主要介面；WebUI 位於 [shittim-chest](https://github.com/celestia-island/shittim-chest)。

僅執行微核心多智慧體平台 —— LLM 僅可呼叫 3 個工具（`exec`、`write_to_var`、`write_to_var_json`）。12 個 Layer1 智慧體，容器隔離的任務執行，IEPL TypeScript 流水線。

## 快速開始

**Linux / macOS：**

```bash
curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
```

**Windows (WSL2)：**

```powershell
irm https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.ps1 | iex
```

**[架構](../../ARCHITECTURE.md)** · **[構建](building.md)** · **[安全](../../SECURITY.md)** · **[文件](./)**

## 許可證

Business Source License 1.1 —— 商業使用需獲取授權許可。非商業使用遵循 SySL-1.0 協定。
