+++
title = "aoba를 통한 시리얼 포트 통신"
description = """아키텍처 결정 기록 —— aoba를 통한 시리얼 포트 통신."""
lang = "ko"
category = "design"
subcategory = "router"
+++

# aoba를 통한 시리얼 포트 통신

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

Evernight는 임베디드 장치 관리와 산업용 프로토콜(Modbus RTU) 탐색을 위한 시리얼 포트 지원이 필요하다. 자매 크레이트 `aoba`는 이미 크로스 플랫폼 시리얼 포트 열거, VID/PID/시리얼 추출, Modbus RTU/TCP 마스터 기능을 제공한다.

## 결정

모든 시리얼 포트 작업을 aoba에 위임한다. Evernight의 `serial` 모듈은 타입(`SerialConfig`, `SerialPortInfo`)을 정의하고 시리얼 전송 위에 `TerminalBackend`를 구현하며, 실제 포트 I/O는 aoba를 호출한다. 프로토콜 자동 감지(Modbus RTU 보레이트/패리티 스윕) 또한 aoba에 위임한다.

## 결과

### 긍정적 측면

- 검증된 코드를 재사용하며, aoba가 크로스 플랫폼 엣지 케이스를 처리한다.

### 부정적 측면

- aoba에 대한 의존성이 추가되며, serial 기능은 aoba가 사용 가능해야 한다.
