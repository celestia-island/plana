+++
title = "SSH 구성 파일 파서"
description = """아키텍처 결정 기록 —— SSH 구성 파일 파서."""
lang = "ko"
category = "design"
subcategory = "router"
+++

# SSH 구성 파일 파서

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

사용자는 `ssh` 명령과 마찬가지로 `evernight`가 호스트 별칭, 점프 호스트, 키 파일을 위해 `~/.ssh/config`를 읽기를 기대한다. 이것이 없으면 사용자는 매번 모든 연결 매개변수를 반복 입력해야 한다.

## 결정

표준 지시어인 `Host`, `HostName`, `User`, `Port`, `IdentityFile`, `ProxyJump`, `ForwardAgent`, `ServerAliveInterval` 등을 처리하는 순수 Rust SSH 구성 파일 파서를 구현한다. `Host` 항목에 대한 Glob 패턴 매칭. 투명한 구성 기반 연결을 위해 연결 풀과 통합된다.

## 결과

### 긍정적 측면

- 드롭인 SSH 호환성; 구성 기반 점프 호스트 해석.

### 부정적 측면

- OpenSSH 구성 문법 변경을 추적해야 함.
- `Match` 지시어 지원이 복잡함.
