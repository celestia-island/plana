+++
title = "핵심 개념"
description = """> 대상: shittim-chest의 설계에 대한 개념적 이해를 원하는 개발자."""
lang = "ko"
category = "guides"
subcategory = "webui"
+++

# 핵심 개념

> **대상**: shittim-chest의 설계에 대한 개념적 이해를 원하는 개발자.
> **최종 업데이트**: 2026-05-25

## 두 저장소 아키텍처

shittim-chest와 [entelecheia](https://github.com/celestia-island/entelecheia)는 의도적인 경계를 가진 두 저장소 시스템을 형성합니다:

- **entelecheia** — 에이전트 오케스트레이션 코어 (scepter, 13개 에이전트, Cosmos/IEPL 런타임). 신원, 권한, 에이전트 설정을 소유.
- **shittim-chest** — 사용자 대면 셸. 인증, 세션, 채팅 데이터, LLM 제공자 설정, 프론트엔드 UI, scepter로의 프록시 브리지를 소유.

이들은 JWT 인증 HTTP 및 WebSocket을 통해 통신합니다. 어느 쪽도 상대방의 데이터베이스에 직접 접근하지 않습니다. 이 분리를 통해 각 저장소가 독립적으로 개발, 배포, 확장될 수 있습니다.

## 이중 운영 모드

shittim-chest는 두 가지 운영 모드를 지원합니다:

### 독립형 모드

자체 LLM 라우팅 계층으로 독립 실행. 지원:

- 스트리밍 응답이 있는 채팅 (SSE + WebSocket)
- 구성된 제공자를 통한 이미지 생성
- 사용자 인증 (비밀번호 + GitHub OAuth)
- 제공자 관리 (LLM 제공자 추가/제거)

entelecheia가 필요하지 않습니다. 개발 및 단순 배포에 유용.

### 프록시 모드

entelecheia의 에이전트 시스템으로의 게이트웨이 역할. 추가:

- JWT 통과가 있는 scepter로의 요청 전달
- 에이전트 기반 채팅을 위한 WebSocket 브리징
- 웹훅 수신 및 트리거 전달
- polemos를 통한 원격 장치 관리
- RBAC 권한 쿼리 및 캐싱

실행 중인 entelecheia 인스턴스 필요. 두 모드는 공존 가능 — 간단한 채팅은 독립형 LLM, 에이전트 오케스트레이션은 프록시.

## 인증 모델

인증은 shittim_chest가 발행한 JWT 토큰을 사용:

1. **자격 증명 저장**: 비밀번호(argon2 해시), 세션, 리프레시 토큰, API 키는 `shittim_chest_db`에 저장.
1. **GitHub OAuth**: 사용자가 GitHub으로 로그인 가능; 첫 로그인 시 계정 자동 생성.
1. **권한 저장**: 사용자 그룹, 역할, 권한 매트릭스는 `entelecheia_db`에 저장.
1. **JWT 흐름**: 로그인 시 shittim_chest가 로컬에서 자격 증명 검증, 그런 다음 scepter에서 권한 조회. 발행된 JWT는 `{ sub: user_id, groups: [...] }` 포함.
1. **공유 비밀키**: JWT 서명 비밀키가 scepter와 공유되어 두 서비스가 독립적으로 토큰 검증 가능.
1. **토큰 순환**: 액세스 토큰은 1시간 후 만료; 리프레시 토큰은 7일. 리프레시 토큰은 각 사용 시 순환.

## 프론트엔드 (webui)

webui는 `packages/webui/`의 통합 프론트엔드로, `/`에 채팅 인터페이스, `/backend`에 관리자 패널이 있으며, Vue 3 + Vite + Pinia (`@vitejs/plugin-vue-jsx`를 통한 TSX)로 구축되었습니다.

## LLM 제공자 시스템

shittim-chest는 독립적인 LLM 라우팅 계층을 보유:

- **제공자**: 구성 가능한 LLM API 엔드포인트 (OpenAI 호환). AES-256-GCM 암호화된 API 키와 함께 `shittim_chest_db`에 저장.
- **라우터**: 우선순위 기반 선택 및 자동 폴백이 있는 다중 제공자 라우팅.
- **카테고리**: 제공자는 `chat`, `image` 또는 둘 다로 태그 가능.
- **관리**: REST API 및 webui 관리자 패널을 통한 전체 CRUD. 제공자 연결 테스트 가능.
- **스트리밍**: SSE (간단, 프록시 친화적) 및 WebSocket (양방향) 스트리밍 프로토콜 모두.

## 채팅 시스템

- **대화**: 제목과 메타데이터가 있는 스레드 기반 채팅 세션
- **메시지**: 텍스트, 이미지, 도구 호출(함수 호출) 지원
- **스트리밍**: SSE 또는 WebSocket을 통한 실시간 토큰별 응답 전달
- **검색**: ILIKE 쿼리를 사용한 전체 텍스트 메시지 검색
- **내보내기**: 대화를 JSON 또는 Markdown 형식으로 내보내기 가능
- **이미지 생성**: 구성된 제공자를 통한 프롬프트 기반 이미지 생성, "채팅에 삽입" 기능 포함

## 원격 장치 관리

shittim-chest는 entelecheia/polemos가 관리하는 원격 장치를 위한 브라우저 기반 인터페이스 제공:

- **데스크톱**: 프레임 릴레이가 있는 WebRTC 기반 원격 데스크톱 뷰어
- **터미널**: WebSocket 릴레이가 있는 xterm.js 기반 터미널 에뮬레이터
- **파일 브라우저**: SFTP 파일 브라우저 백엔드 (골격)
- **시그널링**: WebSocket 기반 WebRTC 시그널링 릴레이 (SDP offer/answer, ICE candidate)

모든 장치 통신은 entelecheia의 polemos 에이전트를 통해 흐릅니다 — shittim-chest는 절대로 엔드포인트에 직접 연결하지 않습니다.

## 프록시 아키텍처

shittim_chest는 사용자와 scepter 사이의 게이트웨이 역할:

- **HTTP 역방향 프록시**: `/api/proxy/*`가 JWT 통과와 함께 인증된 요청을 scepter로 전달.
- **WebSocket 브리지**: 채팅 스트리밍이 양방향 WebSocket 전달 사용 (`브라우저 ↔ shittim_chest ↔ scepter`).

이를 통해 shittim_chest는 scepter가 개별 브라우저 연결을 처리할 필요 없이 속도 제한 집행, 사용량 기록, 연결 생명주기 관리 가능.

## 웹훅 파이프라인

외부 이벤트가 웹훅 파이프라인을 통해 에이전트 코어에 도달:

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC 검증 → 이벤트 파싱 → Unix 소켓을 통해 scepter로 전달 → 에이전트 디스패치
```

각 제공자는 자체 검증 메커니즘 보유:

- **GitHub**: `X-Hub-Signature-256`을 통한 HMAC-SHA256
- **GitLab**: `X-Gitlab-Token`을 통한 토큰
- **Gitee**: 토큰 폴백이 있는 HMAC

추가 기능: 중복 배달 감지 (LRU 캐시), 배달 로깅, IP 화이트리스트, 일반 사용자 정의 웹훅 엔드포인트.

## RBAC 모델

권한은 그룹 기반 RBAC 모델을 따름:

- **그룹**: 사용자는 하나 이상의 그룹에 속함.
- **역할**: 그룹에는 할당된 역할 있음.
- **권한**: 각 역할은 다음을 포함하는 권한 매트릭스 정의:
  - 제공자 할당량 (최대 토큰, 최대 요청)
  - 에이전트 화이트리스트 (그룹이 접근 가능한 에이전트)
  - 관리 기능 (사용자 관리, 제공자 구성)

shittim_chest는 TTL(기본값 5분)로 인프로세스 권한 캐시. 캐시 무효화는 TTL 만료, 로그아웃, 또는 scepter에서 전파된 명시적 권한 변경 시 발생.

## 프론트엔드 전략

shittim-chest는 2단계 프론트엔드 접근 방식 사용:

**1단계 (현재)**: `packages/webui/`의 Vue 3 프론트엔드(`webui`), `@vitejs/plugin-vue-jsx`를 통한 TSX로 Vite + Pinia 사용. API 계약을 정의하고 프로덕션 품질 참조 구현 제공.

**2단계 (미래)**: Tairitsu로 구축된 Rust → WASM 프론트엔드. 레거시 프론트엔드는 살아있는 명세 및 테스트 오라클 역할 — 동일한 사용자 상호작용이 동일한 결과 생성해야 함.

## 타입 안전 브리지

TypeScript 타입은 외부 `arona` 프로토콜 크레이트를 통해 Rust 코드에서 생성되어 프론트엔드-백엔드 일관성 보장:

```text
arona Rust 크레이트 (git 의존성)
  → #[derive(ts_rs::TS)]
  → ts-rs codegen → packages/webui/src/types/arona/ (TypeScript)
  → @celestia-island/arona로 webui에서 소비
```

이것은 수동 타입 동기화를 제거합니다. `arona` 크레이트의 Rust 타입이 변경되면 TypeScript 바인딩이 재생성되어 webui에서 소비됩니다.
