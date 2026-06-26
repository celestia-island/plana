+++
title = "Entelecheia"
description = """基于 Rust 的多智能体协作平台"""
lang = "zhs"
category = "guides"
subcategory = "core"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Entelecheia logo" width="200"/>

# Entelecheia

**基于 Rust 的多智能体协作平台**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fentelecheia-blue.svg)](https://github.com/celestia-island/entelecheia)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **版本 0.2.0** — 早期开发阶段。TUI 是主要界面；WebUI 位于 [shittim-chest](https://github.com/celestia-island/shittim-chest)。

仅执行微内核多智能体平台 —— LLM 仅可调用 3 个工具（`exec`、`write_to_var`、`write_to_var_json`）。12 个 Layer1 智能体，容器隔离的任务执行，IEPL TypeScript 流水线。

## 快速开始

**Linux / macOS：**

```bash
curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
```

**Windows (WSL2)：**

```powershell
irm https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.ps1 | iex
```

**[架构](../../ARCHITECTURE.md)** · **[构建](building.md)** · **[安全](../../SECURITY.md)** · **[文档](./)**

## 许可证

Business Source License 1.1 —— 商业使用需获取授权许可。非商业使用遵循 SySL-1.0 协议。
