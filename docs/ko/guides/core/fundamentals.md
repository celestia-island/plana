+++
title = "기본 개념"
description = """> 현재 코드 현실을 기준으로 한 개념 설명"""
lang = "ko"
category = "guides"
subcategory = "core"
+++

# 기본 개념

> 현재 코드 현실을 기준으로 한 개념 설명

## 개요

Entelecheia(현추)는 더 작은 모델 가시 도구 표면, 공유 런타임 및 여러 클라이언트 진입점을 갖춘 다중 에이전트 플랫폼입니다. 저장소에는 현재 구현, 실험적 기능 및 설계 문서가 함께 포함되어 있으므로, 본 가이드는 현재 코드에서 이미 활성화된 핵심 개념만 설명합니다.

## 핵심 개념

### Agent

Agent는 프롬프트, skill 및 MCP tool을 가진 런타임 역할입니다.

- Layer1은 현재 플랫폼의 핵심 기능입니다.
- 현재 workspace에서 활성화된 내장 Layer2는 Web Automation입니다.
- Layer3(설계 단계)는 `.amphoreus/` 디렉터리에서 로드할 계획입니다 — 아직 구현되지 않았습니다.

### Exec-Only 도구 표면

모델은 모든 MCP 도구를 직접 볼 수 없습니다. 현재 주요 모델 가시 도구는:

- `exec`
- `write_to_var`
- `write_to_var_json`

런타임 내부에서 `exec`의 코드는 ES 모듈 가져오기를 통해 도구 함수를 호출할 수 있습니다(예: `import { tool } from 'agent'`).

### MCP 도구

MCP 도구는 내부의 구조화된 기능 인터페이스입니다.

- 일부는 이미 실제로 구현되었습니다.
- 일부는 부분적으로 구현되었습니다.
- 일부는 여전히 스텁(stub) 또는 매개변수 검증 골격입니다.

따라서 문서에 등장하는 모든 도구를 이미 안정적으로 제공 가능한 것으로 기본적으로 이해해서는 안 됩니다.

### Skill

Skill은 프롬프트 기반으로 정의된 워크플로로, 관련 도구를 참조하며 때로는 다른 skill도 참조합니다.

- 일부 skill은 이미 실제 워크플로를 구동할 수 있습니다.
- 일부 skill은 완전한 자동화 체인보다 SOP 문서에 더 가깝습니다.

### 계층

| 계층 | 현재 의미 |
| --- | --- |
| Layer1 | workspace에서 컴파일 활성화된 핵심 Agent |
| Layer2 | Web Automation이라는 활성 내장 도메인 Agent와 일부 아카이브 설계 |
| Layer3 | 사용자 정의 Agent（계획 중, 아직 구현되지 않음） |

## 클라이언트

### TUI

현재 가장 완전하고 성숙한 사용자 진입점은 TUI입니다.

### WebUI

Web UI(arona 채팅) 및 관리 패널(plana)은 자매 저장소 [shittim-chest](https://github.com/celestia-island/shittim-chest)로 이전되었으며 본 코드베이스에서 제거되었습니다. 본 저장소의 기본 인터페이스는 TUI입니다.

### CLI

CLI는 존재하지만, 일부 명령은 여전히 플레이스홀더 출력입니다.

### Tauri 클라이언트

데스크톱 및 모바일 코드는 이미 [shittim-chest](https://github.com/celestia-island/shittim-chest) 형제 저장소에 존재하지만, 초기 통합으로 보는 것이 더 적합합니다. IDE 통합(VS Code, IntelliJ)도 동일하게 shittim-chest에 위치합니다.

## 보안 모델의 보수적 표현

- JWT 및 API key 인증 기능이 이미 있습니다.
- 알려진 HTTP, WebSocket, MCP 경로에 대한 RBAC 매핑이 이미 있습니다.
- 암호화된 provider key 저장 기능이 이미 있습니다.
- 컨테이너 하드닝과 감사 완전성은 아직 불완전합니다.

구체적인 코드 경로를 확인하지 않았다면, 양방향 TLS, 완전한 capability token 또는 전체 체인 엄격 정책 시행을 현재 사실로 간주해서는 안 됩니다.
