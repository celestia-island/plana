+++
title = "TerminalBackend / ViewportBackend / FileBackend 트레이트 추상화"
description = """아키텍처 결정 기록 —— TerminalBackend / ViewportBackend / FileBackend 트레이트 추상화."""
lang = "ko"
category = "design"
subcategory = "router"
+++

# TerminalBackend / ViewportBackend / FileBackend 트레이트 추상화

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

Evernight는 프런트엔드(CLI, TUI, GUI)가 모든 프로토콜을 일관되게 소비할 수 있도록 다형적 백엔드 인터페이스가 필요하다. 공유 트레이트가 없으면 각 프런트엔드는 터미널, 그래픽 디스플레이, 파일 작업에 대해 프로토콜별 코드 경로를 갖게 된다.

## 결정

크레이트 루트에 세 가지 객체 안전(object-safe) 비동기 트레이트를 정의한다(항상 사용 가능, feature flag 없음):

- **`TerminalBackend`** — `read` / `write` / `resize` / `close`
- **`ViewportBackend`** — `render` / `input` / `clipboard` / `close`
- **`FileBackend`** — `list` / `stat` / `get` / `put` / `rm` / `mkdir` / `rename`

각 프로토콜 백엔드는 관련 트레이트를 구현한다. 프런트엔드는 `Box<dyn TerminalBackend>` 등을 소비한다.

## 결과

### 긍정적 측면

- 프런트엔드는 프로토콜에 구애받지 않으며, 새로운 백엔드(예: RDP)를 프런트엔드 변경 없이 추가할 수 있다.

### 부정적 측면

- 비동기 트레이트 객체는 `Box::pin` 오버헤드를 수반한다.
- 트레이트 설계는 안정적이어야 하며, 변경 시 모든 구현체가 깨진다.
