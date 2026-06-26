+++
title = "Entelecheia"
description = """Plataforma de colaboración multi-agente basada en Rust"""
lang = "es"
category = "guides"
subcategory = "core"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Entelecheia logo" width="200"/>

# Entelecheia

**Plataforma de colaboración multi-agente basada en Rust**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fentelecheia-blue.svg)](https://github.com/celestia-island/entelecheia)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Versión 0.2.0** — Etapa de desarrollo temprano. TUI es la interfaz principal; WebUI está en [shittim-chest](https://github.com/celestia-island/shittim-chest).

Plataforma multi-agente de micronúcleo solo ejecución — el LLM solo puede invocar 3 herramientas (`exec`, `write_to_var`, `write_to_var_json`). 12 agentes Layer1, ejecución de tareas aislada en contenedores, pipeline IEPL TypeScript.

## Inicio rápido

**Linux / macOS：**

```bash
curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
```

**Windows (WSL2)：**

```powershell
irm https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.ps1 | iex
```

**[Arquitectura](../../ARCHITECTURE.md)** · **[Construcción](building.md)** · **[Seguridad](../../SECURITY.md)** · **[Documentación](./)**

## Licencia

Business Source License 1.1 —— el uso comercial requiere autorización. El uso no comercial se rige por la licencia SySL-1.0.
