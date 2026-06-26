+++
title = "Shittim Chest (什亭之匣)"
description = """[entelecheia](https://github.com/celestia-island/entelecheia) 멀티 에이전트 플랫폼을 위한 사용자 대면 셸"""
lang = "ko"
category = "guides"
subcategory = "webui"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="docs/logo.webp" alt="Shittim Chest 로고" width="200"/>

# Shittim Chest (什亭之匣)

**[entelecheia](https://github.com/celestia-island/entelecheia) 멀티 에이전트 플랫폼을 위한 사용자 대면 셸**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fshittim--chest-blue.svg)](https://github.com/celestia-island/shittim-chest)

**[English](README.md)** &bull; **[简体中文](docs/guides/zhs/README.md)** &bull;
**[繁體中文](docs/guides/zht/README.md)** &bull; **[日本語](docs/guides/ja/README.md)** &bull;
**[한국어](docs/guides/ko/README.md)** &bull; **[Français](docs/guides/fr/README.md)** &bull;
**[Español](docs/guides/es/README.md)** &bull; **[Русский](docs/guides/ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **버전 0.1.0** — 활발한 개발 중.

[Entelecheia](https://github.com/celestia-island/entelecheia) 멀티 에이전트 플랫폼을 위한 Webui, 백엔드, CLI. 채팅, 관리자 패널, 인증, 다중 채널 통합, 장치 관리를 포함합니다.

## 빠른 시작

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
just dev    # 백엔드 :3000, 프론트엔드 :5173
```

**전제 조건**: Rust 1.85+, Node 20+, pnpm 9+, [just](https://github.com/casey/just), PostgreSQL 18+.

**[아키텍처](ARCHITECTURE.md)** · **[기여하기](CONTRIBUTING.md)** · **[보안](SECURITY.md)** · **[문서](docs/guides/en/)**

## 라이선스

Business Source License 1.1 — 상업적 사용에는 라이선스가 필요합니다. 비상업적 사용은 Synthetic Source License (SySL-1.0)에 따릅니다; 2030-01-01에 SySL-1.0으로 완전히 전환됩니다.
