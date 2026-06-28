+++
title = "빌드 및 개발 가이드"
description = """> 대상: 로컬 shittim-chest 개발 환경을 설정하는 기여자."""
lang = "ko"
category = "guides"
subcategory = "webui"
+++

# 빌드 및 개발 가이드

> **대상**: 로컬 shittim-chest 개발 환경을 설정하는 기여자.
> **최종 업데이트**: 2026-05-25

## 전제 조건

| 도구 | 최소 버전 | 비고 |
| --- | --- | --- |
| Rust | 1.85+ | Edition 2024 필수; <https://rustup.rs>를 통해 설치 |
| Node.js | 20+ | LTS 권장 |
| pnpm | 9+ | `corepack enable && corepack prepare pnpm@latest --activate` |
| just | 최신 | 명령 실행기; `cargo install just` |
| PostgreSQL | 18+ | 인증 + 채팅 데이터용 shittim_chest_db |
| entelecheia scepter | 선택 사항 | 프록시/장치 기능에 필요; 독립형 채팅에는 선택 사항 |

모든 것 확인:

```bash
rustc --version    # >= 1.85
node --version     # >= 20
pnpm --version     # >= 9
just --version
psql --version     # >= 18
```

## 클론 및 부트스트랩

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
```

## 환경 변수

클론 후 `.env`를 편집하십시오. 모든 변수는 인라인으로 문서화되어 있습니다; 아래는 요약입니다.

### 서버

| 변수 | 기본값 | 목적 |
| --- | --- | --- |
| `SHITTIM_CHEST_HOST` | `0.0.0.0` | 수신 주소 |
| `SHITTIM_CHEST_PORT` | `80` | 수신 포트 |

### 데이터베이스

| 변수 | 기본값 | 목적 |
| --- | --- | --- |
| `SHITTIM_CHEST_DATABASE_URL` | `postgresql://sc:pass@localhost:5432/shittim_chest` | PostgreSQL 연결 문자열 |
| `SHITTIM_CHEST_DATABASE_MAX_CONNECTIONS` | `10` | SeaORM 연결 풀 크기 |

데이터베이스와 사용자 생성:

```sql
CREATE USER sc WITH PASSWORD 'pass';
CREATE DATABASE shittim_chest OWNER sc;
```

### JWT 및 암호화

| 변수 | 기본값 | 목적 |
| --- | --- | --- |
| `JWT_SECRET` | `change-me-in-production` | scepter와 공유되는 비밀키; **일치해야 함** |
| `JWT_EXPIRATION_SECONDS` | `3600` | 액세스 토큰 수명 (1시간) |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | 리프레시 토큰 수명 (7일) |
| `SHITTIM_CHEST_ENCRYPTION_KEY` | `change-me-32-bytes-base64-encoded` | API 키 및 OAuth 토큰용 AES-256-GCM 키 |

프로덕션 키 생성:

```bash
openssl rand -base64 32
```

### LLM 제공자 (독립형 운영용)

entelecheia 없이 shittim-chest를 독립적으로 사용하려면 다음을 설정:

| 변수 | 목적 |
| --- | --- |
| `LLM_DEFAULT_PROVIDER_ENDPOINT` | OpenAI 호환 API 엔드포인트 (예: `https://api.deepseek.com/v1`) |
| `LLM_DEFAULT_PROVIDER_API_KEY` | 제공자의 API 키 |
| `LLM_DEFAULT_PROVIDER_MODELS` | 쉼표로 구분된 모델 목록 (예: `deepseek-chat,deepseek-reasoner`) |
| `LLM_DEFAULT_PROVIDER_CATEGORY` | 제공자 카테고리: `chat` 또는 `image` |
| `LLM_STREAM_BUFFER_SECONDS` | 스트림 버퍼 타임아웃 (기본값: 60) |
| `LLM_MAX_TOKENS_DEFAULT` | 기본 최대 토큰 (기본값: 4096) |
| `LLM_REQUEST_TIMEOUT_SECONDS` | HTTP 요청 타임아웃 (기본값: 120) |

### 원격 장치

| 변수 | 기본값 | 목적 |
| --- | --- | --- |
| `REMOTE_DEVICES_ENABLED` | `false` | 원격 장치 기능 활성화 |
| `REMOTE_DEVICES_SCEPTER_SOCK` | `/run/entelecheia/device_stream.sock` | 장치 데이터용 Unix 소켓 |
| `REMOTE_DEVICES_FRAME_BUFFER_SIZE` | `4194304` | 프레임 버퍼 크기 (바이트) |
| `REMOTE_DEVICES_MAX_SESSIONS_PER_USER` | `3` | 최대 동시 장치 세션 |
| `WEBRTC_ICE_SERVERS` | `stun:stun.l.google.com:19302` | ICE 서버 목록 |

### GitHub OAuth

| 변수 | 목적 |
| --- | --- |
| `GITHUB_CLIENT_ID` | GitHub OAuth 앱 클라이언트 ID |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth 앱 클라이언트 비밀키 |
| `GITHUB_REDIRECT_URI` | OAuth 콜백 URL (예: `https://your-domain/api/auth/github/callback`) |

### Scepter 연결 (프록시 기능용)

| 변수 | 기본값 | 목적 |
| --- | --- | --- |
| `ENTELECHEIA_SCEPTER_URL` | `http://localhost:8424` | scepter용 HTTP 엔드포인트 |
| `ENTELECHEIA_SCEPTER_WS_URL` | `ws://localhost:8424` | scepter용 WebSocket 엔드포인트 |
| `ENTELECHEIA_TUI_SOCK` | `/run/entelecheia/entelecheia.sock` | 트리거 전달용 Unix 소켓 |

### 웹훅

| 변수 | 목적 |
| --- | --- |
| `WEBHOOK_GITHUB_SECRET` | GitHub 웹훅 검증용 HMAC 비밀키 |
| `WEBHOOK_GITLAB_SECRET` | GitLab 웹훅 검증용 토큰 |
| `WEBHOOK_PUBLIC_URL` | 웹훅 엔드포인트용 공개 URL |

## 데이터베이스 설정

```bash
just db-init      # 스키마 생성 (SeaORM 마이그레이션 실행)
just db-migrate   # 보류 중인 마이그레이션 적용
```

### 스키마 개요

shittim_chest_db는 사용자 대면 데이터를 소유합니다:

| 테이블 | 목적 |
| --- | --- |
| `auth_users` | argon2 비밀번호 해시가 있는 사용자 계정 |
| `sessions` | 리프레시 토큰이 있는 활성 세션 |
| `api_keys` | API 키 기록 (해시됨) |
| `oauth_connections` | 제3자 OAuth 바인딩 (GitHub) |
| `conversations` | 채팅 대화 |
| `messages` | 도구 호출 데이터가 있는 채팅 메시지 |
| `llm_providers` | LLM 제공자 설정 (API 키 암호화) |
| `remote_devices` | 원격 장치 기록 |
| `device_sessions` | 활성 장치 세션 |
| `channel_configs` | 다중 플랫폼 채널 설정 |
| `channel_messages` | 채널 메시지 기록 |
| `channel_pairings` | 채널-채팅 페어링 |

데이터베이스 재설정:

```bash
just db-reset
```

## 백엔드 개발

```bash
just dev-backend
```

이것은 `cargo run --package shittim_chest`를 실행합니다. 서버는 `:80`에서 시작됩니다.

### CLI 명령

```bash
shittim_chest db-init      # 데이터베이스 스키마 생성
shittim_chest db-migrate   # 보류 중인 마이그레이션 적용
shittim_chest db-reset     # 스키마 삭제 및 재생성
shittim_chest server       # 웹 서버 시작
```

### 핫 리로드

```bash
cargo install cargo-watch
cargo watch -x 'run --package shittim_chest -- server'
```

### API 엔드포인트 개요

| 라우트 그룹 | 목적 |
| --- | --- |
| `/api/auth/*` | 로그인, 등록, GitHub OAuth, 리프레시, 로그아웃 |
| `/api/chat/*` | 대화, 메시지, SSE/WS 스트리밍, 검색, 내보내기 |
| `/api/providers/*` | LLM 제공자 CRUD, API 키 관리, 테스트 |
| `/api/generation/*` | 이미지 생성, 모델 목록 |
| `/api/devices/*` | 원격 장치 목록, 세션, WebRTC 시그널링 |
| `/api/webhook/*` | GitHub/GitLab/Gitee/사용자 정의 웹훅 수신 |
| `/api/proxy/*` | scepter로의 역방향 프록시 (HTTP + WebSocket) |
| `/static/*` | SPA 정적 파일 호스팅 |

## 프론트엔드 개발

### 의존성 설치

```bash
pnpm install
```

### webui

```bash
just dev    # 프론트엔드 빌드 + :3000에서 백엔드 시작
just watch  # 파일 변경 시 자동 재빌드
```

두 프론트엔드 모두 Vite에 의해 `dist/`로 빌드됩니다. 백엔드는 이러한 정적 파일을 `:3000`에서 직접 제공합니다 — 별도의 Vite 개발 서버나 프록시가 필요하지 않습니다. 개발 모드에서 `dev.py`는 프론트엔드 소스를 감시하고 자동으로 재빌드합니다.

## 프로젝트 간 설정

공유 `arona` 프로토콜 크레이트를 사용한 로컬 개발의 경우, 로컬 체크아웃으로 패치하십시오. `~/.cargo/config.toml` 편집 (저장소에 절대 커밋하지 마십시오):

```toml
[patch.'https://github.com/celestia-island/arona']
arona = { path = "/path/to/arona" }
```

npm의 경우, webui는 `@celestia-island/arona` 경로 별칭을 통해 `arona` 크레이트의 TS 바인딩을 소비하며, 이는 `packages/webui/src/types/arona/`를 가리킵니다.

## 프로덕션 빌드

```bash
just build
```

이것은 `cargo build --release` 및 `pnpm run build:all`을 실행합니다. 출력 위치:

- 백엔드 바이너리: `target/release/shittim_chest`
- 프론트엔드 자산: `packages/webui/dist/`

### Docker

CLI 래퍼로 빌드 및 실행 (Docker API 직접 사용):

```bash
just dev
```

또는 수동으로:

```bash
just build        # Docker 이미지 빌드
just up           # 모든 서비스 시작
just migrate      # 데이터베이스 마이그레이션 실행
```

프로덕션 바이너리는 Axum의 정적 파일 미들웨어를 통해 `/`에서 프론트엔드 자산을 제공합니다. 별도의 프론트엔드 서버가 필요하지 않습니다.

## 일반적인 문제

### 데이터베이스 연결 거부

```text
error: connection to server at "localhost", port 5432 failed
```

**해결**: PostgreSQL이 실행 중이고 `.env`의 `SHITTIM_CHEST_DATABASE_URL`이 설정과 일치하는지 확인하십시오. `psql "$SHITTIM_CHEST_DATABASE_URL" -c 'SELECT 1'`로 확인.

### Scepter에 연결할 수 없음

```text
error: error sending request for url (http://localhost:8424/...)
```

**해결**: entelecheia scepter 인스턴스를 시작하거나, LLM 제공자가 구성된 독립형 모드를 사용하십시오. 백엔드는 채팅/이미지 생성을 위해 scepter 없이도 작동합니다.

### 브라우저에서 CORS 오류

```text
Access-Control-Allow-Origin header is present on the requested resource
```

**해결**: 개발 백엔드는 `localhost` 오리진에 대해 CORS를 활성화합니다. 포트를 변경한 경우 CORS 설정을 업데이트하십시오. 프로덕션 배포는 CORS를 처리하도록 역방향 프록시(nginx/caddy)를 구성해야 합니다.

### pnpm install 실패

**해결**: pnpm 9+를 사용 중인지 확인하십시오. `corepack enable && corepack prepare pnpm@latest --activate`를 실행하여 올바른 버전을 설정하십시오.

### 공유 크레이트에서 cargo build 실패

**해결**: `~/.cargo/config.toml`에 로컬 패치가 있는 경우, 경로가 존재하고 크레이트 이름이 일치하는지 확인하십시오. 대신 git 의존성을 사용하려면 패치 섹션을 제거하십시오.
