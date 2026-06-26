+++
title = "Entelecheia"
description = """A Rust-based multi-agent collaboration platform"""
lang = "en"
category = "guides"
subcategory = "core"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Entelecheia logo" width="200"/>

# Entelecheia

**A Rust-based multi-agent collaboration platform**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fentelecheia-blue.svg)](https://github.com/celestia-island/entelecheia)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.2.0** — Early development stage. The TUI is the primary interface; the WebUI is located at [shittim-chest](https://github.com/celestia-island/shittim-chest).

An exec-only microkernel multi-agent platform — the LLM can only invoke 3 tools (`exec`, `write_to_var`, `write_to_var_json`). 12 Layer1 agents, container-isolated task execution, and an IEPL TypeScript pipeline.

## Quick Start

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
```

**Windows (WSL2):**

```powershell
irm https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.ps1 | iex
```

**[Architecture](../../ARCHITECTURE.md)** · **[Building](building.md)** · **[Security](../../SECURITY.md)** · **[Docs](./)**

## License

Business Source License 1.1 — commercial use requires an authorization license. Non-commercial use follows the SySL-1.0 protocol.
