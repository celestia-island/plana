+++
title = "Evernight"
description = """Evernight — bibliothèque et démon de contrôle à distance multiplateforme."""
lang = "fr"
category = "guides"
subcategory = "router"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Evernight logo" width="200"/>

# Evernight

**Aucune machine hors de portée — Bibliothèque et démon de contrôle à distance multiplateforme**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fevernight-blue.svg)](https://github.com/celestia-island/evernight)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — Développement préliminaire.

Contrôle à distance multiplateforme : capture d'écran, streaming WebRTC, shell distant SSH, transfert de fichiers, télémétrie matérielle, sondage de protocoles industriels (Modbus, OPC-UA, MQTT), tunneling TCP, traversée NAT. Utilisable comme bibliothèque Rust ou démon autonome.

## Démarrage rapide

```bash
cargo build --release

# Lister le matériel
./target/release/evernight --list-hardware

# Commande SSH distante
./target/release/evernight --remote-exec "uname -a" --shell-host 192.168.1.100 --shell-user root --shell-key ~/.ssh/id_ed25519

# Tunnel TCP
./target/release/evernight --forward-tunnel 8080:internal.example.com:80
```

**[Pour commencer](getting-started.md)** · **[ADR](../../design/en/)** · **[Documentation](./)**

## Licence

MIT
