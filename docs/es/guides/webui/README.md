+++
title = "Shittim Chest (什亭之匣)"
description = """Interfaz de usuario para la plataforma multi-agente [entelecheia](https://github.com/celestia-island/entelecheia)"""
lang = "es"
category = "guides"
subcategory = "webui"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Logo de Shittim Chest" width="200"/>

# Shittim Chest (什亭之匣)

**Interfaz de usuario para la plataforma multi-agente [entelecheia](https://github.com/celestia-island/entelecheia)**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fshittim--chest-blue.svg)](https://github.com/celestia-island/shittim-chest)

**[English](README.md)** &bull; **[简体中文](docs/guides/zhs/README.md)** &bull;
**[繁體中文](docs/guides/zht/README.md)** &bull; **[日本語](docs/guides/ja/README.md)** &bull;
**[한국어](docs/guides/ko/README.md)** &bull; **[Français](docs/guides/fr/README.md)** &bull;
**[Español](docs/guides/es/README.md)** &bull; **[Русский](docs/guides/ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Versión 0.1.0** — En desarrollo activo.

Webui, backend y CLI para la plataforma multi-agente [Entelecheia](https://github.com/celestia-island/entelecheia). Incluye chat, panel de administración, autenticación, integraciones multicanal y gestión de dispositivos.

## Inicio Rápido

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
just dev    # backend en :3000, frontend en :5173
```

**Requisitos previos**: Rust 1.85+, Node 20+, pnpm 9+, [just](https://github.com/casey/just), PostgreSQL 18+.

**[Arquitectura](ARCHITECTURE.md)** · **[Contribuir](CONTRIBUTING.md)** · **[Seguridad](SECURITY.md)** · **[Documentación](docs/guides/en/)**

## Licencia

Business Source License 1.1 — el uso comercial requiere una licencia. Uso no comercial bajo la Synthetic Source License (SySL-1.0); se convierte completamente a SySL-1.0 el 2030-01-01.
