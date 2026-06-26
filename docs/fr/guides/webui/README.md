+++
title = "Shittim Chest (什亭之匣)"
description = """Coque utilisateur pour la plateforme multi-agent [entelecheia](https://github.com/celestia-island/entelecheia)"""
lang = "fr"
category = "guides"
subcategory = "webui"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Logo Shittim Chest" width="200"/>

# Shittim Chest (什亭之匣)

**Coque utilisateur pour la plateforme multi-agent [entelecheia](https://github.com/celestia-island/entelecheia)**

[![Licence](https://img.shields.io/badge/licence-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fshittim--chest-blue.svg)](https://github.com/celestia-island/shittim-chest)

**[English](README.md)** &bull; **[简体中文](docs/guides/zhs/README.md)** &bull;
**[繁體中文](docs/guides/zht/README.md)** &bull; **[日本語](docs/guides/ja/README.md)** &bull;
**[한국어](docs/guides/ko/README.md)** &bull; **[Français](docs/guides/fr/README.md)** &bull;
**[Español](docs/guides/es/README.md)** &bull; **[Русский](docs/guides/ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — Développement actif.

Webui, backend et CLI pour la plateforme multi-agent [Entelecheia](https://github.com/celestia-island/entelecheia). Inclut le chat, le panneau d'administration, l'authentification, les intégrations multi-canaux et la gestion de périphériques.

## Démarrage Rapide

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
just dev    # backend sur :3000, frontend sur :5173
```

**Prérequis** : Rust 1.85+, Node 20+, pnpm 9+, [just](https://github.com/casey/just), PostgreSQL 18+.

**[Architecture](ARCHITECTURE.md)** · **[Contribuer](CONTRIBUTING.md)** · **[Sécurité](SECURITY.md)** · **[Docs](docs/guides/en/)**

## Licence

Business Source License 1.1 — l'usage commercial nécessite une licence. Usage non commercial sous la Synthetic Source License (SySL-1.0) ; passe entièrement à SySL-1.0 le 2030-01-01.
