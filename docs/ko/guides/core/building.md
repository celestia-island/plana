+++
title = "빌드 가이드"
description = """- [전제 조건](#전제-조건)"""
lang = "ko"
category = "guides"
subcategory = "core"
+++

# 빌드 가이드

---

## 목차

- [전제 조건](#전제-조건)
- [설치](#설치)
- [구성](#구성)
- [빌드](#빌드)
- [실행](#실행)
- [데이터베이스 관리](#데이터베이스-관리)
- [개발 환경](#개발-환경)
- [배포](#배포)
- [문제 해결](#문제-해결)
- [Webhook 봇 실행](#webhook-봇-실행)

---

## 전제 조건

### 시스템 요구 사항

- **운영체제**: Linux, macOS 또는 Windows(Docker CLI 필요)
- **메모리**: 최소 8GB, 16GB 권장
- **저장소**: 최소 20GB 사용 가능 공간
- **CPU**: 4코어 이상 권장

> 설명(설계 의도)
> Windows 측의 핵심 요구 사항은 Docker CLI를 사용할 수 있어야 하며, 명령은 PowerShell 또는 Windows Terminal에서 직접 실행할 수 있어야 합니다.
> 그러나 컨테이너는 궁극적으로 Linux 런타임이 필요합니다:
> 1. 로컬 방안은 일반적으로 Docker Desktop(일반적으로 WSL2 백엔드에 의존)입니다.
> 2. 대체 방안으로, 로컬에 Docker CLI만 설치하고 `docker context`를 통해 원격 Linux Docker 호스트로 전달할 수 있습니다.

### 소프트웨어 요구 사항

#### 필수 소프트웨어

- **Docker 또는 Podman**(컨테이너 런타임 환경)

```bash
docker --version
docker compose version
```

현재 플랫폼에 따라 공식 권장 설치 방식을 사용하십시오:

- Linux: Docker Engine, Docker Desktop for Linux, 또는 배포판 자체 Podman 설치
- macOS: Docker Desktop 또는 Podman Desktop 설치
- Windows: Docker Desktop 또는 Podman Desktop 설치

**중요 설명**:

- PostgreSQL 등 런타임 종속성은 컨테이너화된 환경에 포함되어 있습니다
- 그러나 `just` 레시피나 저장소 내 보조 스크립트를 실행하려면 호스트에 Python 3.8+가 여전히 필요합니다
- 호스트에 PostgreSQL을 별도로 설치할 필요는 없습니다
- Windows에서 명령은 PowerShell 또는 Windows Terminal에서 직접 실행할 수 있지만, 배포에는 사용 가능한 Docker/Podman Linux 런타임이 여전히 필요합니다. 로컬 배포는 일반적으로 WSL2 백엔드를 갖춘 Docker Desktop을 사용하는 것을 의미합니다. 로컬 Docker CLI/context를 통해 원격 Linux Docker 호스트로 전달할 수도 있습니다.

- **Rust 1.85+**(개발 빌드에만 필요)

```bash
rustup update stable
```

플랫폼에 맞는 공식 rustup 설치 방식을 사용하십시오:

- Linux/macOS: <https://rustup.rs> 방문
- Windows: <https://rustup.rs>에서 `rustup-init.exe`를 다운로드하여 실행한 후 `rustup update stable` 실행

#### 권장 소프트웨어

- **just**(명령 러너)

```bash
  # cargo 사용
  cargo install just

  # brew 사용(macOS)
  brew install just
  ```

- **VS Code** 및 rust-analyzer 확장 설치

---

## 설치

### 단계 1: 저장소 클론

```bash
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia
```

### 단계 2: 환경 변수 구성

```bash
# .env.example에서 .env를 생성한 후 구성 편집
nano .env  # 또는 선호하는 편집기 사용
```

현재 셸 또는 파일 관리자를 사용하여 `.env.example`을 `.env`로 복사하십시오.

POSIX shell:

```bash
cp .env.example .env
```

PowerShell:

```powershell
Copy-Item .env.example .env
```

#### 기본 구성

```bash
# 데이터베이스 구성(컨테이너 내부에서 자동 구성됨)
# DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
# DATABASE_MAX_CONNECTIONS=10

# LLM 빠른 초기화, 시작 후 ApoRia 가져오기
# 단일 provider:
# LLM_API_KEY=your-api-key-here
# LLM_BASE_URL=https://api.openai.com/v1
# LLM_MODEL=gpt-4
# 다중 provider(세미콜론으로 구분):
# LLM_API_KEY=key1;key2
# LLM_BASE_URL=https://api.one/v1;https://api.two/v1
# LLM_PROTOCOL=openai;openai,api-key
# LLM_MODEL_DEEP=model-a1,model-a2;model-b1
# LLM_MODEL_NORMAL=model-a3;model-b2
# LLM_MODEL_BASIC=model-a4;model-b3

# provider 수준 바로가기(권장)
OPENAI_API_KEY=your-api-key-here
# ANTHROPIC_API_KEY=
# DEEPSEEK_API_KEY=
# DASHSCOPE_API_KEY=
# BIGMODEL_API_KEY=
# ZAI_API_KEY=

# WebSocket 구성
WS_BIND_ADDRESS=127.0.0.1:42470
WS_MAX_CONNECTIONS=100
```

#### LLM 환경 변수 구성 설명

> **중요 안내**: 현재 LLM provider 구성은 ApoRia가 통합 관리합니다. 환경 변수는 시작 부트스트랩 진입점으로만 사용되며, 더 이상 장기 구성 소스가 아닙니다.

**작동 방식**:

1. TUI가 자동으로 서버를 시작해야 할 때, 일반 `LLM_*` 빠른 초기화 변수 또는 `OPENAI_API_KEY`와 같은 provider 수준 변수를 읽습니다. 다중 provider 구성은 세미콜론으로 구분된 병렬 배열을 사용합니다: `LLM_API_KEY`, `LLM_BASE_URL`, `LLM_PROTOCOL`, `LLM_MODEL_DEEP`, `LLM_MODEL_NORMAL`, `LLM_MODEL_BASIC`. 프로그래밍 패키지 환경 변수(예: `BIGMODEL_API_KEY_CODING_PRO`)도 세미콜론으로 여러 키를 구분할 수 있으며, 자동으로 `(#2)`, `(#3)`로 번호가 매겨집니다. 사용자 정의 provider는 괄호 안에 도메인 이름이 표시됩니다.
1. 서버 시작 전에 TUI는 먼저 첫 번째 provider 구성을 `res/prompts/agents/aporia/config.toml`에 미리 작성합니다
1. 미리 작성이 완료되면 ApoRia 구성과 TUI의 Models 페이지를 기준으로 합니다
1. 이미 존재하고 API key가 비어 있지 않은 provider는 환경 변수로 덮어쓰이지 않습니다

**권장 사용법**:

- 환경 변수를 사용하여 최초 부트스트랩 완료
- 이후에는 Models 페이지 또는 `res/prompts/agents/aporia/config.toml`을 통해 통합 관리

### 단계 3: 서비스 시작

```bash
# Docker Compose를 사용하여 모든 서비스 시작
docker compose up -d

# 또는 just 명령 사용(설치된 경우)
just dev
```

---

## 구성

### LLM 제공자 구성

Entelecheia(현추)는 여러 LLM 제공자를 지원합니다. 선호하는 제공자를 구성하십시오:

#### OpenAI

```bash
OPENAI_API_KEY=sk-...
```

#### Anthropic

```bash
ANTHROPIC_API_KEY=sk-ant-...
```

#### 로컬 LLM(Ollama)

```bash
# Models 페이지 또는 res/prompts/agents/aporia/config.toml을 통해 로컬 provider 구성
# endpoint = http://localhost:11434
# model = llama2
```

### Docker 구성

```bash
# Docker socket(일반적으로 자동 감지)
DOCKER_HOST=unix:///var/run/docker.sock

# 컨테이너 설정
CONTAINER_NETWORK=entelecheia-network
CONTAINER_REGISTRY=127.0.0.1:5000
```

---

## 빌드

### 개발 빌드

```bash
# 빠른 개발 빌드
just build-dev
```

### 프로덕션 빌드

```bash
# 최적화된 릴리스 빌드
just build
```

### 특정 컴포넌트 빌드

```bash
# 서버만 빌드
cargo build -p scepter

# TUI만 빌드
cargo build -p entelecheia-tui

# 특정 에이전트 빌드
cargo build -p haplotes
```

### 빌드 산출물

빌드 완료 후 다음을 찾을 수 있습니다:

- **바이너리 파일**: `target/debug/` 또는 `target/release/`
- **Docker 이미지**: `just dev` 실행 중 자동 빌드

---

## 실행

### 개발 모드

```bash
# 전체 개발 환경 시작(TUI 포함)
just dev

# 서버만 시작(TUI 없음)
just dev --no-tui

# 클린 시작(모든 데이터 삭제)
just dev-clean
```

### 프로덕션 모드

```bash
# 서버 시작
just server

# TUI 클라이언트 시작
just tui

# 모든 에이전트 시작
just agents-up
```

### 터미널 호환성 매개변수

TUI는 ANSI 이스케이프 시퀀스, 마우스 이벤트 및 이미지 렌더링(Sixel/Kitty 프로토콜)에 의존합니다. SSH 세션, 직렬 콘솔, CI 러너 또는 구형 터미널 에뮬레이터와 같은 제한된 터미널 환경에서는 세 가지 점진적 성능 저하 매개변수를 사용할 수 있습니다:

#### `--no-image-render`

모든 이미지 렌더링을 비활성화합니다. 나머지 기능(색상, 마우스, 차등 새로 고침)은 완전히 정상 작동합니다.

```bash
just tui -- --no-image-render
```

적용 시나리오: 터미널이 색상과 마우스를 지원하지만 Sixel/Kitty 이미지 프로토콜 지원이 부족한 경우(가장 일반적인 경우).

#### `--no-ansi`

마우스 캡처 및 특수 키 수신을 비활성화합니다. 색상과 차등(부분) 화면 새로 고침은 유지됩니다. 마우스 이벤트가 터미널 선택, 복사-붙여넣기 또는 스크롤백 기록을 방해할 때 유용합니다.

```bash
just tui -- --no-ansi
```

적용 시나리오: 색상은 필요하지만 마우스 캡처가 문제를 일으키는 경우(터미널 멀티플렉서, `screen`, 기본 `tmux` 구성 등).

#### `--no-ansi-pure`

순수 단색 모드 — 가장 공격적인 성능 저하. 모든 ANSI 색상 비활성화(전역적으로 `Color::Reset` 강제), 마우스 캡처 비활성화, 매 프레임 전체 화면 다시 그리기. 시작 화면 Logo는 순수 ASCII 아트 버전으로 대체됩니다. 이 매개변수는 `--no-ansi`를 암시합니다.

```bash
just tui -- --no-ansi-pure
```

적용 시나리오: 최소한의 터미널 지원을 통한 SSH, 직렬 콘솔, `docker exec`, CI 환경에서 실행하거나, ANSI 색상 코드를 올바르게 처리할 수 없는 모든 터미널.

#### 매개변수 비교

| 기능 | 기본값 | `--no-image-render` | `--no-ansi` | `--no-ansi-pure` |
| --- | --- | --- | --- | --- |
| 색상 | 전체 | 전체 | 전체 | 비활성화 |
| 마우스 캡처 | 예 | 예 | 아니오 | 아니오 |
| 이미지 렌더링 | 예 | 아니오 | 아니오 | 아니오 |
| 화면 새로 고침 | 차등 | 차등 | 차등 | 전체 다시 그리기 |
| 시작 Logo | ANSI 색상 | ANSI 색상 | ANSI 색상 | 순수 ASCII 아트 |

### 서비스 관리

```bash
# 서비스 상태 확인
just dev-status

# 로그 보기
just dev-logs

# 서비스 중지
just dev-down

# 모든 서비스 강제 종료
just dev-kill
```

---

## 데이터베이스 관리

### 데이터베이스 초기화

```bash
# 데이터베이스 생성
just db-create

# 마이그레이션 실행
just db-migrate

# 시드 데이터로 초기화
just db-init
```

### 데이터베이스 작업

```bash
# 데이터베이스 상태 확인
just db-status

# 데이터베이스 백업
just db-backup

# 데이터베이스 복원
just db-restore backups/backup_xxx.sql

# 데이터베이스 초기화(경고: 모든 데이터 삭제)
just db-reset
```

### 마이그레이션 관리

```bash
# 새 마이그레이션 생성
cargo test -p scepter test_create_migration -- --nocapture --ignored

# 마지막 마이그레이션 롤백
just db-migrate-down
```

---

## 개발 환경

### 환경 설정

```bash
# 모든 종속성 초기화
just init

# Python 종속성 확인

# 코드 포맷팅
just fmt

# 코드 검사 실행
just clippy
```

### 테스트

```bash
# 모든 테스트 실행
just test

# 특정 유형의 테스트 실행
just test unit
just test integration
just test e2e
just test llm-providers

# 상세 출력
just test verbose
```

### 코드 품질

```bash
# 코드 포맷팅
just fmt

# 포맷 확인
just fmt-check

# clippy 실행
just clippy

# 타입 검사
just check
```

---

## 배포

### Docker 배포

#### 이미지 빌드

```bash
docker build -t entelecheia:latest .
```

#### 컨테이너 실행

```bash
docker run -d --name entelecheia \
  --env-file .env \
  -p 8424:8424 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  entelecheia:latest
```

### Docker Compose 배포

```bash
# 모든 서비스 시작
docker compose up -d

# 로그 보기
docker compose logs -f

# 서비스 중지
docker compose down
```

---

## 문제 해결

### 자주 묻는 문제

#### Docker 권한 거부

```bash
# 사용자를 docker 그룹에 추가
sudo usermod -aG docker $USER

# 로그아웃 후 다시 로그인
```

#### 포트가 이미 사용 중

```bash
# 포트를 점유 중인 프로세스 확인
lsof -i :8424

# 프로세스 종료
kill -9 <PID>
```

#### 빌드 실패

```bash
# 빌드 산출물 정리
cargo clean

# 종속성 업데이트
cargo update

# 다시 빌드
just build
```

#### 컨테이너 시작 불가

```bash
# Docker 로그 확인
docker compose logs

# 컨테이너 다시 빌드
docker compose down
docker compose build --no-cache
docker compose up -d
```

### 도움말 얻기

1. [GitHub Issues](https://github.com/celestia-island/entelecheia/issues) 검색
1. [토론 게시판](https://github.com/celestia-island/entelecheia/discussions) 참여

---

## Webhook 봇 실행

Webhook 봇은 `plugins/github-webhook/` 아래에 위치합니다. 각 플랫폼마다 독립된 디렉터리가 있습니다.

### 전제 조건

- Python 3.10+(현재 봇)
- Node.js 18+(향후 TypeScript 마이그레이션)
- 각 플랫폼의 bot token([Webhook 구성 가이드](webhook-setup.md) 참조)

### 단일 봇 실행

```bash
# GitHub
cd plugins/github-webhook/github
pip install -r requirements.txt
python bot.py

# Gitee
cd plugins/github-webhook/gitee
pip install -r requirements.txt
python bot.py

# Discord
cd plugins/github-webhook/discord
pip install -r requirements.txt
python bot.py
```

### 모든 봇 실행

```bash
just webhooks-up
```

### 환경 변수

예제 환경 파일을 복사하여 구성하십시오:

```bash
cp plugins/github-webhook/.env.example plugins/github-webhook/.env
```

각 플랫폼의 구체적인 구성 세부 사항은 [Webhook 구성 가이드](webhook-setup.md)를 참조하십시오.

---

## 다음 단계

- [기초 가이드](fundamentals.md)를 읽어 아키텍처 이해하기
- [에이전트 문서](../../agents/)를 탐색하여 사용 가능한 에이전트 확인하기

---

**즐거운 빌드 되세요!** 🚀
