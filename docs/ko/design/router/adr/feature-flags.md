+++
title = "Feature Flag 아키텍처"
description = """아키텍처 결정 기록 —— Feature Flag 아키텍처."""
lang = "ko"
category = "design"
subcategory = "router"
+++

# Feature Flag 아키텍처

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

Evernight의 단일체 의존성 그래프는 모든 소비자가 SSH나 하드웨어 텔레메트리만 필요하더라도 모든 의존성(webrtc, russh, screenshots, sysinfo)을 가져오도록 강제한다. 이는 하위 사용자에게 컴파일 시간과 바이너리 크기를 증가시킨다.

## 결정

Cargo feature flag를 사용하여 크레이트를 다음 기능들로 분할한다:

| 기능         | 게이트 대상                                     |
|--------------|--------------------------------------------------|
| `screen`     | 화면 캡처 모듈 + `screenshots` 크레이트          |
| `webrtc`     | WebRTC 모듈 + `webrtc` 크레이트 (`screen` 암시) |
| `remote-ssh` | SSH 핸들러 모듈 + `russh` 크레이트               |
| `hardware`   | 하드웨어 텔레메트리 모듈 + `sysinfo` 크레이트    |
| `protocol`   | 프로토콜/메시지 타입 (무거운 의존성 없음)        |
| `tunnel`     | TCP 터널 모듈                                    |
| `full`       | 모든 기능 (기본값)                               |

각 feature는 모듈과 그 의존성을 모두 게이트한다. `webrtc` 기능은 WebRTC 세션이 화면 캡처를 요구하므로 `screen`을 암시한다.

## 결과

### 긍정적 측면

- 소비자는 필요한 것만 컴파일한다
- 부분 사용 시 컴파일 시간 단축

### 부정적 측면

- Feature flag 조합 행렬이 커지며, CI에서 각 feature 조합을 테스트해야 한다
