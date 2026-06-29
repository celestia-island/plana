+++
title = "연결 항목 URI 스킴"
description = """아키텍처 결정 기록 —— 연결 항목 URI 스킴."""
lang = "ko"
category = "design"
subcategory = "router"
+++

# 연결 항목 URI 스킴

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

연결 카탈로그는 여러 프로토콜 연결(SSH, VNC, RDP, 시리얼, Docker)을 일관되게 표현할 방법이 필요하다. URI 스킴은 사람이 읽을 수 있고 직렬화 가능한 기술자를 제공한다.

## 결정

프로토콜별 URI 스킴을 사용한다:

- `ssh://user@host:port`
- `vnc://host:5900`
- `rdp://host:3389`
- `serial:///dev/ttyUSB0?baud=9600`
- `docker:///var/run/docker.sock?container=name`

`ConnectionEntry`는 URI를 파싱하여 scheme, host, port, username, path, query 파라미터를 가진 타입 구조체로 변환한다. 카탈로그는 항목을 담고 있는 `ConnectionCategory` 노드의 트리 구조이다.

## 결과

### 긍정적 측면

- 익숙한 URI 형식; 직렬화가 용이하며 연결 정보의 복사-붙여넣기 공유를 지원한다.

### 부정적 측면

- 일부 프로토콜은 URI로 깔끔하게 매핑되지 않는다(예: Kubernetes 컨텍스트).
- 쿼리 스트링 파라미터가 구조화되어 있지 않다.
