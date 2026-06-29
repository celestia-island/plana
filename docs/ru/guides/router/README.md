+++
title = "Evernight"
description = """Evernight — кроссплатформенная библиотека и демон удалённого управления."""
lang = "ru"
category = "guides"
subcategory = "router"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Evernight logo" width="200"/>

# Evernight

**Ни одна машина не останется недосягаемой — Кроссплатформенная библиотека и демон удалённого управления**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fevernight-blue.svg)](https://github.com/celestia-island/evernight)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — Ранняя стадия разработки.

Кроссплатформенное удалённое управление: захват экрана, потоковая передача WebRTC, удалённая оболочка SSH, передача файлов, аппаратная телеметрия, зондирование промышленных протоколов (Modbus, OPC-UA, MQTT), туннелирование TCP, обход NAT. Используется как библиотека Rust или автономный демон.

## Быстрый старт

```bash
cargo build --release

# Список оборудования
./target/release/evernight --list-hardware

# Удалённая SSH-команда
./target/release/evernight --remote-exec "uname -a" --shell-host 192.168.1.100 --shell-user root --shell-key ~/.ssh/id_ed25519

# TCP-туннель
./target/release/evernight --forward-tunnel 8080:internal.example.com:80
```

**[Начало работы](getting-started.md)** · **[ADR](../../design/en/)** · **[Документация](./)**

## Лицензия

MIT
