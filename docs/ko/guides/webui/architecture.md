+++
title = "아키텍처 심층 분석"
description = """> 대상: shittim-chest의 내부 작동 방식을 이해해야 하는 개발자."""
lang = "ko"
category = "guides"
subcategory = "webui"
+++

# 아키텍처 심층 분석

> **대상**: shittim-chest의 내부 작동 방식을 이해해야 하는 개발자.
> **최종 업데이트**: 2026-05-25

## 프로젝트 개요

shittim-chest는 Rust 기반 멀티 에이전트 협업 플랫폼인 [entelecheia](https://github.com/celestia-island/entelecheia)의 **사용자 대면 셸**입니다. 경계는 의도적입니다:

- **entelecheia**는 에이전트 오케스트레이션(scepter, 13개 에이전트, Cosmos/IEPL 런타임), 신원, 권한을 소유합니다.
- **shittim-chest**는 사용자 인증, 세션 관리, 채팅 데이터, LLM 제공자 설정, 프론트엔드 표현, scepter로의 프록시 브리지를 소유합니다.

이들은 JWT 인증 HTTP 및 WebSocket을 통해 통신합니다. shittim-chest는 에이전트 작업을 위해 entelecheia의 데이터베이스에 직접 접근하지 않습니다.

## 백엔드 스택

### Axum 라우터

코어 백엔드(`packages/core`)는 Axum 0.8 애플리케이션입니다. 라우터는 다음 모듈 그룹을 마운트합니다:

```text
/                   → 상태 확인
/api/auth/*         → AuthService (로그인, 등록, GitHub OAuth, 리프레시, 로그아웃)
/api/chat/*         → ChatService (대화, 메시지, SSE/WS 스트리밍, 검색, 내보내기)
/api/providers/*    → ProviderService (LLM 제공자 CRUD, API 키 암호화, 테스트)
/api/generation/*   → GenerationService (이미지 생성)
/api/devices/*      → DeviceService (원격 장치 목록, 세션, 시그널링)
/api/webhook/*      → WebhookService (GitHub, GitLab, Gitee, 사용자 정의; HMAC 검증)
/api/proxy/*        → ProxyService (HTTP 역방향 프록시 + scepter로의 WebSocket 브리지)
/static/*           → SPA 정적 호스팅 (프로덕션 전용)
```

### SeaORM + PostgreSQL

데이터베이스 접근은 PostgreSQL과 함께 SeaORM 1.x를 사용합니다. `shittim_chest_db`는 다음을 저장합니다:

- 사용자 인증: 비밀번호 해시(argon2), 세션, 리프레시 토큰, API 키, OAuth 연결
- 채팅 데이터: 대화, 메시지
- LLM 제공자 설정 (API 키는 AES-256-GCM으로 저장 시 암호화)
- 원격 장치 기록 및 장치 세션
- 다중 플랫폼 메시징을 위한 채널 설정
- 웹훅 배달 로그

5개 마이그레이션과 25개 엔티티 모델이 `packages/core/src/{migration,entity}/`에 있습니다.

### JWT 인증

shittim_chest는 `{ sub: user_id, groups: [...] }`를 포함한 JWT를 발행합니다. JWT 비밀키는 scepter와 공유되므로 두 서비스가 독립적으로 토큰을 검증할 수 있습니다. 액세스 토큰은 1시간 후 만료; 리프레시 토큰은 7일이며 각 사용 시 순환됩니다.

## 독립적 LLM 기능

shittim-chest는 entelecheia와 독립적으로 작동하는 자체 LLM 라우팅 계층을 가지고 있습니다:

- **LlmRouter**: 우선순위 기반 선택 및 폴백이 있는 다중 제공자 라우터
- **제공자 관리**: LLM 제공자 추가/편집/제거를 위한 CRUD 엔드포인트
- **API 키 암호화**: 제공자 API 키는 AES-256-GCM으로 저장 시 암호화
- **OpenAI 호환**: 모든 OpenAI 호환 API와 작동 (DeepSeek, OpenAI, 로컬 모델 등)
- **이중 스트리밍**: 채팅 응답을 위한 SSE (Server-Sent Events) 및 WebSocket 스트리밍

즉, shittim-chest는 entelecheia 없이 독립형 채팅 애플리케이션으로 실행하거나, 프록시 계층을 통해 entelecheia 에이전트를 사용할 수 있습니다.

## 인증 흐름

### 로그인 순서

```text
사용자 → shittim_chest: POST /api/auth/login { username, password }
shittim_chest → shittim_chest_db: SELECT user WHERE username = ? (argon2 해시 검증)
shittim_chest → scepter: GET /api/user/{id}/permissions
scepter → entelecheia_db: 그룹 + 권한 쿼리
scepter → shittim_chest: { groups: [...], permissions: {...} }
shittim_chest → 사용자: { access_token, refresh_token }
shittim_chest: 세션 저장 + RBAC 캐시
```

### GitHub OAuth

```text
사용자 → shittim_chest: GET /api/auth/github
shittim_chest → 사용자: 302 GitHub OAuth로 리다이렉트
사용자 → GitHub: 승인
GitHub → shittim_chest: GET /api/auth/github/callback?code=...
shittim_chest → GitHub: 액세스 토큰으로 코드 교환
shittim_chest → GitHub: GET /user (사용자 정보 조회)
shittim_chest → shittim_chest_db: oauth_connections INSERT/UPDATE
shittim_chest → 사용자: { access_token, refresh_token } (신규 사용자 자동 생성)
```

## 채팅 아키텍처

### 메시지 흐름 (독립형 LLM)

```text
사용자 → POST /api/chat/conversations/:id/messages
shittim_chest: JWT 검증, 대화 로드
shittim_chest → LlmRouter: 최적 제공자로 요청 라우팅
LlmRouter → LLM 제공자: POST chat/completions (스트리밍)
LLM 제공자 → LlmRouter: SSE 스트림
LlmRouter → 사용자: SSE/WS 스트림 (토큰 도착 시)
shittim_chest: shittim_chest_db에 메시지 저장
```

### SSE vs WebSocket 스트리밍

- **SSE** (`/api/chat/stream`): 간단한 HTTP 스트리밍, 프록시 통과, 자동 재연결
- **WebSocket** (`/ws/chat/stream`): 양방향, 취소 및 실시간 상호작용 지원

## 프록시 아키텍처

`/api/proxy/*` 엔드포인트는 인증된 요청을 scepter로 전달합니다:

1. 브라우저가 JWT와 함께 `ws://shittim-chest:80/api/proxy/chat` 열기
1. shittim_chest가 JWT 검증, JWT를 전달하는 scepter 연결 열기
1. 브라우저와 scepter 간 양방향 메시지 전달
1. shittim_chest가 속도 제한 집행, 사용량 기록, 연결 생명주기 관리

## 웹훅 파이프라인

외부 서비스의 웹훅이 `/api/webhook/*`를 통해 진입합니다:

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → HMAC 검증 → 이벤트 파싱 → Unix 소켓을 통해 scepter로 전달
```

지원 소스: GitHub (HMAC-SHA256), GitLab (토큰), Gitee (HMAC + 토큰 폴백), 일반 `/api/webhook/custom/{name}` 엔드포인트 포함. 기능:

- 중복 배달 감지 (LRU 캐시, 10,000 ID)
- 목록 API가 있는 배달 로그
- 웹훅 소스를 위한 IP 화이트리스트

## 원격 장치 관리

원격 장치는 시그널링 릴레이를 통해 관리됩니다:

```text
브라우저 (webui) → WS /api/devices/stream → shittim_chest (시그널 릴레이) → Unix 소켓 → entelecheia/polemos
```

기능:

- REST를 통한 장치 목록 및 세션 CRUD
- WebRTC 시그널링 (SDP offer/answer, ICE candidate)
- 터미널 릴레이 (WebSocket에서 xterm.js로)
- 데스크톱 프레임 릴레이
- SFTP 파일 브라우저 백엔드

shittim-chest는 원격 장치에 직접 연결하지 않습니다 — 모든 데이터는 entelecheia의 polemos 에이전트를 통해 흐릅니다.

## 데이터 소유권

### shittim_chest_db

| 데이터 | 테이블 | 근거 |
| --- | --- | --- |
| 비밀번호 해시 (argon2) | `auth_users` | 표현 계층이 로그인 흐름 소유 |
| 활성 세션, 리프레시 토큰 | `sessions` | 세션 관리는 프론트엔드 관심사 |
| 암호화된 API 키 | `api_keys` | API 키 발행은 사용자 대면 |
| OAuth 연결 | `oauth_connections` | 제3자 인증 바인딩은 사용자 대면 |
| 대화, 메시지 | `conversations`, `messages` | 채팅 데이터는 사용자 대면 |
| LLM 제공자 설정 | `llm_providers` | 제공자 관리는 사용자 대면 (키 암호화) |
| 원격 장치 기록 | `remote_devices`, `device_sessions` | 장치 추적은 사용자 대면 |
| 채널 설정 | `channel_configs` 등 | 다중 플랫폼 설정은 사용자 대면 |

### entelecheia_db

| 데이터 | 근거 |
| --- | --- |
| 사용자 신원, 그룹, 역할 할당 | 코어가 권한 집행 |
| GroupPermissions (제공자 할당량, 에이전트 화이트리스트) | 에이전트 수준 정책은 에이전트와 함께 |
| 에이전트 설정, Cosmos/IEPL 상태 | 오케스트레이션 데이터는 코어에 속함 |

## 이중 프론트엔드 전략

### 1단계: Vue 3 (현재)

| 패키지 | 기술 | 포트 | 목적 |
| --- | --- | --- | --- |
| `webui` | Vue 3 + Vite + Pinia (TSX) | `:3000 (공유)` | 통합 webui: 채팅, 이미지 생성, 장치, 관리자 (제공자, 에이전트, RBAC, 웹훅) |

### 2단계: Rust WASM (미래)

| 패키지 | 기술 | 목적 |
| --- | --- | --- |
| `webui` | Rust → WASM (Tairitsu) | 장기 통합 webui (채팅 + 관리자) |

레거시 프론트엔드는 살아있는 명세로 사용됩니다. 전환 기간 동안 두 버전이 병렬 실행되며, 동일한 사용자 상호작용이 동일한 결과를 생성해야 합니다.

## 역방향 프록시 배포 모드

shittim-chest는 `.env`의 `SHITTIM_CHEST_PROXY_MODE`로 제어되는 세 가지 역방향 프록시 모드를 지원합니다.

### 모드 1: None (직접)

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=none   # 또는 미설정
```

코어 서버가 `SHITTIM_CHEST_HOST:SHITTIM_CHEST_PORT`(기본값 `0.0.0.0:80`)에 직접 바인딩됩니다. TLS 없음, 역방향 프록시 컨테이너 없음. 다음에 적합:

- 로컬 개발
- 기존 역방향 프록시 뒤 (Cloudflare Tunnel, AWS ALB, Traefik 레이블)
- 다른 서비스가 TLS 종료를 처리하는 Docker 네트워크

### 모드 2: Caddy 자동

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_DOMAIN=app.example.com
```

CLI가 `shittim-chest-caddy` 컨테이너(이미지 `caddy:2`)를 생성하여:

1. 포트 80/443에서 수신 (`SHITTIM_CHEST_PROXY_HTTP_PORT` / `SHITTIM_CHEST_PROXY_HTTPS_PORT`로 설정 가능)
1. Let's Encrypt를 통해 TLS 인증서 자동 프로비저닝 (Caddy 내장 ACME)
1. 모든 요청을 Docker 네트워크의 코어 백엔드로 프록시

Caddyfile이 필요하지 않습니다 — CLI가 자동 생성합니다. 도메인은 호스트를 가리키는 공용 DNS가 있어야 합니다.

### 모드 3: Caddy 사용자 정의

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/caddy/Caddyfile
SHITTIM_CHEST_PROXY_EXTRA_VOLUMES=/etc/letsencrypt:/etc/letsencrypt
```

동일한 Caddy 컨테이너이지만, 직접 Caddyfile을 제공합니다(호스트에서 마운트). 다음이 필요할 때 사용:

- 여러 가상 호스트
- 사용자 정의 TLS 인증서 경로
- 추가 미들웨어 (기본 인증, 속도 제한 등)
- API와 함께 정적 파일 제공

### 모드 4: Nginx 사용자 정의

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=nginx
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/nginx/conf.d/default.conf
```

설정 파일과 함께 `nginx:bookworm` 컨테이너 생성. TLS 인증서는 직접 관리. Nginx가 표준인 환경에 적합.

### 컨테이너 생명주기

모든 프록시 컨테이너는 Docker API (`bollard`)를 통해 CLI로 관리됩니다:

| 명령 | 동작 |
| --- | --- |
| `just dev` / `chest up` | `PROXY_MODE`가 설정된 경우 프록시 컨테이너 생성/시작 |
| `just dev-stop` / `chest down` | 프록시 컨테이너 중지 및 제거 |
| 컨테이너가 이미 실행 중 | 기존 컨테이너 재사용 (멱등) |

프록시 컨테이너는 코어 백엔드와 동일한 Docker 네트워크에 참여하므로, 내부 호스트명(`core` 또는 `shittim-chest`)을 통해 백엔드에 도달합니다.
