+++
title = "Entelecheia"
description = """Rust 기반 다중 에이전트 협업 플랫폼"""
lang = "ko"
category = "guides"
subcategory = "core"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Entelecheia logo" width="200"/>

# Entelecheia

**Rust 기반 다중 에이전트 협업 플랫폼**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fentelecheia-blue.svg)](https://github.com/celestia-island/entelecheia)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **버전 0.2.0** — 초기 개발 단계. TUI가 주요 인터페이스입니다. WebUI는 [shittim-chest](https://github.com/celestia-island/shittim-chest)에 위치합니다.

실행 전용 마이크로커널 다중 에이전트 플랫폼 — LLM은 3개의 도구(`exec`, `write_to_var`, `write_to_var_json`)만 호출할 수 있습니다. 12개의 Layer1 에이전트, 컨테이너 격리 작업 실행, IEPL TypeScript 파이프라인.

## 빠른 시작

**Linux / macOS：**

```bash
curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
```

**Windows (WSL2)：**

```powershell
irm https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.ps1 | iex
```

**[아키텍처](../../ARCHITECTURE.md)** · **[빌드](building.md)** · **[보안](../../SECURITY.md)** · **[문서](./)**

## 라이선스

Business Source License 1.1 — 상업적 사용은 라이선스 허가를 받아야 합니다. 비상업적 사용은 SySL-1.0 프로토콜을 따릅니다.
