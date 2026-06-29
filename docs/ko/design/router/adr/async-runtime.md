+++
title = "비동기 런타임 — tokio"
description = """아키텍처 결정 기록 —— 비동기 런타임 — tokio."""
lang = "ko"
category = "design"
subcategory = "router"
+++

# 비동기 런타임 — tokio

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

Evernight는 네트워크 I/O(SSH, WebRTC, TCP 터널), 타이머 기반 작업(프레임 캡처 루프, 프로토콜 타임아웃), 동시 태스크 관리를 위한 비동기 런타임이 필요하다. Rust 생태계는 tokio, async-std, smol을 제공한다.

## 결정

`tokio`를 유일한 비동기 런타임으로 사용한다. tokio는 Rust 비동기 생태계의 사실상 표준이며 가장 풍부한 드라이버 지원을 갖추고 있다. 주요 의존성(russh, webrtc, reqwest)이 이미 tokio를 요구한다. 동기 컨텍스트에서의 스폰에는 `tokio::runtime::Handle::current()`를 사용한다.

## 결과

### 긍정적 측면

- 생태계 호환성; 런타임 브리징 불필요

### 부정적 측면

- tokio는 무거운 의존성이다
- tokio를 지원하지 않는 async-std 또는 smol 크레이트를 사용할 수 없다
