+++
title = "Entelecheia"
description = """Plateforme de collaboration multi-agent basée sur Rust"""
lang = "fr"
category = "guides"
subcategory = "core"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Entelecheia logo" width="200"/>

# Entelecheia

**Plateforme de collaboration multi-agent basée sur Rust**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fentelecheia-blue.svg)](https://github.com/celestia-island/entelecheia)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.2.0** — Phase de développement précoce. Le TUI est l'interface principale ; la WebUI se trouve dans [shittim-chest](https://github.com/celestia-island/shittim-chest).

Plateforme multi-agent à micro-noyau en exécution seule — le LLM ne peut appeler que 3 outils (`exec`, `write_to_var`, `write_to_var_json`). 12 agents Layer1, exécution des tâches isolée par conteneur, pipeline IEPL TypeScript.

## Démarrage rapide

**Linux / macOS :**

```bash
curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
```

**Windows (WSL2) :**

```powershell
irm https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.ps1 | iex
```

**[Architecture](../../ARCHITECTURE.md)** · **[Construction](building.md)** · **[Sécurité](../../SECURITY.md)** · **[Documentation](./)**

## Licence

Business Source License 1.1 — L'utilisation commerciale nécessite une licence d'autorisation. L'utilisation non commerciale est régie par la licence SySL-1.0.
