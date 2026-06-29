+++
title = "Evernight"
description = """Evernight —— 跨平台遠端控制函式庫與常駐程式。"""
lang = "zht"
category = "guides"
subcategory = "router"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Evernight logo" width="200"/>

# Evernight

**沒有機器不可及 — 跨平台遠端控制庫與守護進程**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fevernight-blue.svg)](https://github.com/celestia-island/evernight)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — 早期開發階段。

跨平台遠端控制：螢幕擷取、WebRTC 串流、SSH 遠端 Shell、檔案傳輸、硬體遙測、工業協定探測（Modbus、OPC-UA、MQTT）、TCP 隧道、NAT 穿透。可作為 Rust 函式庫或獨立守護程序使用。

## 快速開始

```bash
cargo build --release

# 列出硬體
./target/release/evernight --list-hardware

# 遠端 SSH 命令
./target/release/evernight --remote-exec "uname -a" --shell-host 192.168.1.100 --shell-user root --shell-key ~/.ssh/id_ed25519

# TCP 隧道
./target/release/evernight --forward-tunnel 8080:internal.example.com:80
```

**[入門指南](getting-started.md)** · **[ADR](../../design/en/)** · **[文件](./)**

## 授權條款

MIT
