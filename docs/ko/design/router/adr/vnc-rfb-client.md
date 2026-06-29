+++
title = "VNC (RFB) 프로토콜 클라이언트"
description = """아키텍처 결정 기록 —— VNC (RFB) 프로토콜 클라이언트."""
lang = "ko"
category = "design"
subcategory = "router"
+++

# VNC (RFB) 프로토콜 클라이언트

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

Evernight는 VNC를 지원하는 그래픽 원격 데스크톱 클라이언트가 필요하다. RFB 프로토콜(RFC 6143)은 VNC의 표준이다. 선택지: 기존 Rust 크레이트 사용, libvncclient 바인딩, 또는 처음부터 구현.

## 결정

순수 Rust RFB 003.008 클라이언트를 처음부터 구현한다. 버전 핸드셰이크, 보안 협상(None, VncAuth), DES 챌린지-응답 인증, 픽셀 포맷 협상, Raw/CopyRect 인코딩을 지원한다. 프런트엔드 통합을 위해 `ViewportBackend` 트레이트를 구현한다.

## 결과

### 긍정적 측면

- C 의존성 없음; 프로토콜 흐름에 대한 완전한 제어.

### 부정적 측면

- ZRLE 및 Tight 인코딩이 아직 구현되지 않음(구현 부담이 큼).
- VNC 인증을 위해 DES 구현이 필요했음.
