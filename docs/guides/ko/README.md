<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Arona logo" width="200"/>

# Arona

**Entelecheia 멀티 에이전트 플랫폼을 위한 공유 JSON-RPC 2.0 프로토콜 타입**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **버전 0.1.0** — [entelecheia](https://github.com/celestia-island/entelecheia) 및 [shittim-chest](https://github.com/celestia-island/shittim-chest)에서 사용됩니다.

JSON-RPC 2.0 메시지 타입, Agent 분류 체계(14가지 변형), 약 100개의 WebSocket 매개변수 타입. `ts-rs`를 통한 TypeScript 바인딩 자동 생성.

## 사용법

**Rust:**
```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript:**
```bash
pnpm add @celestia-island/arona
```

## 라이선스

Business Source License 1.1 — 비상업적 사용은 Apache-2.0 또는 MIT에 따릅니다.
