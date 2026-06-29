+++
title = "Evernight"
description = """Evernight —— 跨平台远程控制库与守护进程。"""
lang = "zhs"
category = "guides"
subcategory = "router"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Evernight logo" width="200"/>

# Evernight

**没有机器不可及 — 跨平台远程控制库与守护进程**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fevernight-blue.svg)](https://github.com/celestia-island/evernight)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — 早期开发阶段。

跨平台远程控制：屏幕捕获、WebRTC 流媒体、SSH 远程 Shell、文件传输、硬件遥测、工业协议探测（Modbus、OPC-UA、MQTT）、TCP 隧道、NAT 穿透。可作为 Rust 库或独立守护进程使用。

## 快速开始

```bash
cargo build --release

# 列出硬件
./target/release/evernight --list-hardware

# 远程 SSH 命令
./target/release/evernight --remote-exec "uname -a" --shell-host 192.168.1.100 --shell-user root --shell-key ~/.ssh/id_ed25519

# TCP 隧道
./target/release/evernight --forward-tunnel 8080:internal.example.com:80
```

**[入门指南](getting-started.md)** · **[ADR](../../design/en/)** · **[文档](./)**

## 许可证

MIT
