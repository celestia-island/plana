+++
title = "Evernight"
description = """Evernight — biblioteca y demonio de control remoto multiplataforma."""
lang = "es"
category = "guides"
subcategory = "router"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Evernight logo" width="200"/>

# Evernight

**Ninguna máquina fuera de alcance — Biblioteca y demonio de control remoto multiplataforma**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fevernight-blue.svg)](https://github.com/celestia-island/evernight)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — Desarrollo temprano.

Control remoto multiplataforma: captura de pantalla, streaming WebRTC, shell remoto SSH, transferencia de archivos, telemetría de hardware, sondeo de protocolos industriales (Modbus, OPC-UA, MQTT), túnel TCP, traversal NAT. Úselo como biblioteca Rust o demonio independiente.

## Inicio rápido

```bash
cargo build --release

# Listar hardware
./target/release/evernight --list-hardware

# Comando SSH remoto
./target/release/evernight --remote-exec "uname -a" --shell-host 192.168.1.100 --shell-user root --shell-key ~/.ssh/id_ed25519

# Túnel TCP
./target/release/evernight --forward-tunnel 8080:internal.example.com:80
```

**[Primeros pasos](getting-started.md)** · **[ADR](../../design/en/)** · **[Documentación](./)**

## Licencia

MIT
