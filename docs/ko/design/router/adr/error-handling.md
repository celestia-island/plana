+++
title = "오류 처리 — thiserror와 크레이트 Result"
description = """아키텍처 결정 기록 —— 오류 처리 — thiserror와 크레이트 Result."""
lang = "ko"
category = "design"
subcategory = "router"
+++

# 오류 처리 — thiserror와 크레이트 Result

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

Evernight는 모든 모듈(화면, SSH, 하드웨어, 네트워킹, 터널링)에 걸쳐 통합된 오류 타입이 필요하다. 라이브러리는 내부 오류 처리를 편리하게 유지하면서도 깔끔한 오류 API를 노출해야 한다.

## 결정

`thiserror`를 사용하여 각 variant에 `#[error(...)]` 표시 구현과 함께 `EvernightError` 열거형을 파생한다. 크레이트 전역 결과 타입으로 `pub type Result<T> = std::result::Result<T, EvernightError>`를 정의한다. 각 variant는 내부 의존성 세부 사항을 유출하지 않도록 외부 오류 타입을 래핑하는 대신 도메인별 컨텍스트(ScreenCapture, Ssh, Tunnel 등)를 String으로 캡처한다.

## 결과

### 긍정적 측면

- 안정적인 공개 오류 API; 소비자는 내부 구현을 알지 못해도 열거형 variant에 매칭할 수 있다.

### 부정적 측면

- `String` 변환으로 인한 일부 정보 손실
- 구조화된 오류 체이닝 없음
