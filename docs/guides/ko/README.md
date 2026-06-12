<!-- markdownlint-disable MD033 MD041 MD036 -->
<div align="center">

# Arona

**Entelecheia 멀티 에이전트 플랫폼을 위한 공유 JSON-RPC 2.0 프로토콜 타입**

[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/github-celestia--island%2Farona-blue.svg)](https://github.com/celestia-island/arona)

**[English](../../README.md)** &bull; **[简体中文](../zhs/README.md)** &bull;
**[繁體中文](../zht/README.md)** &bull; **[日本語](../ja/README.md)** &bull;
**[한국어](README.md)** &bull; **[Français](../fr/README.md)** &bull;
**[Español](../es/README.md)** &bull; **[Русский](../ru/README.md)**

</div>
<!-- markdownlint-enable MD033 MD041 MD036 -->

## Arona란

Arona은 entelecheia 에이전트 오케스트레이션 코어와 사용자 인터페이스 계층(TUI, CLI, Web UI, IDE 플러그인, Tauri 앱) 간의 통신 프로토콜을 정의합니다. 포함:

- **JSON-RPC 2.0** 메시지 타입
- **에이전트 분류** — `Agent` 열거형 (14개 변형)
- **WebSocket 파라미터 타입** — 스트리밍, 스냅샷, 패치, 태스크, 프로바이더 설정, 지식 베이스, YOLO 순항 제어, WebRTC 시그널링, 아비터, VM 스냅샷을 포괄하는 약 100개의 구조체
- **TypeScript 타입 생성** — 모든 타입이 `ts-rs`를 통해 `bindings/WsTypes.ts`로 내보내집니다

### 이름 유래

Arona (アロナ) — 시킴 궤집 안에서 미션을 조율하고 명령을 라우팅하는 AI 어시스턴트.

## 사용법

**Rust:**

```toml
[dependencies]
arona = { git = "https://github.com/celestia-island/arona.git", branch = "master" }
```

**TypeScript (pnpm / npm):**

```bash
pnpm add @celestia-island/arona
```

```ts
import type { Agent, TuiAgentInfo, SkillStage } from "@celestia-island/arona";
```

## 기여

이슈와 풀 리퀘스트를 환영합니다.

## 라이선스

Business Source License 1.1, Apache-2.0 / MIT 듀얼 패스: 개인, 학술, 비상업적 사용은 Apache 2.0 또는 MIT이 적용됩니다. 상업적 사용(호스팅, 재판매, 유료 서비스)에는 BUSL 라이선스가 필요합니다.

번역: [简体中文](../../LICENSE.zhs) · [繁體中文](../../LICENSE.zht) · [Español](../../LICENSE.es) · [Français](../../LICENSE.fr) · [Русский](../../LICENSE.ru) · [العربية](../../LICENSE.ar)
