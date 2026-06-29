+++
title = "Evernight"
description = """Evernight —— 크로스 플랫폼 원격 제어 라이브러리 및 데몬."""
lang = "ko"
category = "guides"
subcategory = "router"
+++

<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

<img src="../../logo.webp" alt="Evernight logo" width="200"/>

# Evernight

**어떤 기계든 닿을 수 있다 — 크로스 플랫폼 원격 제어 라이브러리 및 데몬**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Fevernight-blue.svg)](https://github.com/celestia-island/evernight)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](../ko/README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

> **Version 0.1.0** — 초기 개발 단계.

크로스 플랫폼 원격 제어: 화면 캡처, WebRTC 스트리밍, SSH 원격 셸, 파일 전송, 하드웨어 원격 측정, 산업용 프로토콜 프로빙(Modbus, OPC-UA, MQTT), TCP 터널링, NAT 트래버설. Rust 라이브러리 또는 독립 실행형 데몬으로 사용 가능.

## 빠른 시작

```bash
cargo build --release

# 하드웨어 목록
./target/release/evernight --list-hardware

# 원격 SSH 명령
./target/release/evernight --remote-exec "uname -a" --shell-host 192.168.1.100 --shell-user root --shell-key ~/.ssh/id_ed25519

# TCP 터널
./target/release/evernight --forward-tunnel 8080:internal.example.com:80
```

**[시작하기](getting-started.md)** · **[ADR](../../design/en/)** · **[문서](./)**

## 라이선스

MIT
