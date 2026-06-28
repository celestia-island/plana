+++
title = "CLI 사용 가이드"
description = """`entelecheia-cli`는 Entelecheia(현추) 다중 에이전트 협업 플랫폼의 명령줄 인터페이스입니다. Unix socket JSON-RPC를 통해 scepter 서버와 통신하며, 채팅 상호작용, 서비스 수명 주기 관리, 에이전트 제어, 구성 등의 기능을 제공합니다."""
lang = "ko"
category = "guides"
subcategory = "core"
+++

# CLI 사용 가이드

`entelecheia-cli`는 Entelecheia(현추) 다중 에이전트 협업 플랫폼의 명령줄 인터페이스입니다. Unix socket JSON-RPC를 통해 scepter 서버와 통신하며, 채팅 상호작용, 서비스 수명 주기 관리, 에이전트 제어, 구성 등의 기능을 제공합니다.

> 설명: CLI는 현재 TUI와 완전히 동등한 기능을 갖추지 못했습니다. 현재 상태는 [ARCHITECTURE.md](../../ARCHITECTURE.md)를 참조하십시오.

---

## 목차

- [설치](#설치)
- [기본 사용법](#기본-사용법)
- [전역 옵션](#전역-옵션)
- [채팅 명령](#채팅-명령)
- [에이전트 관리](#에이전트-관리)
- [서비스 수명 주기](#서비스-수명-주기)
- [구성](#구성)
- [연결 컨텍스트](#연결-컨텍스트)
- [상태 및 모니터링](#상태-및-모니터링)
- [구독 (Layer3)](#구독-layer3)
- [에이전트 실행](#에이전트-실행)
- [타임라인](#타임라인)
- [Docker 이미지](#docker-이미지)
- [고급 사용법](#고급-사용법)

---

## 설치

### 소스에서 빌드

```bash
# 저장소 클론
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia

# CLI 바이너리 빌드
cargo build --package entelecheia-cli

# 또는 just 사용
just cli
```

바이너리 파일은 `target/debug/entelecheia-cli`(debug) 또는 `target/release/entelecheia-cli`(release)에 위치합니다.

### 사전 빌드된 바이너리

사전 빌드된 바이너리는 [GitHub Releases](https://github.com/celestia-island/entelecheia/releases)에서 얻을 수 있습니다. 플랫폼에 맞는 압축 파일을 다운로드하고 바이너리를 `PATH`에 추가하십시오.

---

## 기본 사용법

```bash
# 도움말 표시
entelecheia-cli --help

# 스킬 체인을 통해 메시지 보내기
entelecheia-cli send 이 프로젝트의 아키텍처를 설명해 줘

# 파이프를 통해 메시지 보내기
echo "이 파일을 요약해 줘" | entelecheia-cli send

# 시스템 상태 확인
entelecheia-cli status
```

---

## 전역 옵션

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | 로그 레벨(trace, debug, info, warn, error) | `warn` |
| `-d, --daemon` | 백그라운드로 명령을 디스패치한 후 즉시 종료 | — |
| `-c, --clean` | Cosmos 컨테이너 및 socket 파일 정리 | — |
| `-a, --auto-approve` | 작업 자동 승인(서버가 실행 중인지 확인) | — |
| `-t, --table` | 사람이 읽을 수 있는 테이블 출력(ANSI 형식) | 기본값 |
| `-j, --json` | JSON 출력(기계 판독 가능) | — |
| `-r, --raw` | 원시 일반 텍스트 출력(서식 없음) | — |
| `--format <FORMAT>` | 출력 형식(table, json, raw) | `table` |

출력 형식 옵션:

- `table` — 사람이 읽을 수 있는 테이블 출력
- `json` — 기계 판독 가능 JSON 출력

**예시:**

```bash
# 컨테이너 정리
entelecheia-cli --clean

# JSON 형식으로 상태 가져오기
entelecheia-cli status --format json

# 디버그 모드로 메시지 보내기
entelecheia-cli -l debug send "연결 문제 디버깅"

# 백그라운드 모드로 agent 실행(즉시 반환)
entelecheia-cli -d run my-agent --ci
```

---

## 채팅 명령

`chat` 하위 명령은 세션 에이전트 시스템과의 대화 상호작용을 관리합니다.

### 메시지 보내기

```bash
entelecheia-cli chat send [OPTIONS]
```

| 옵션 | 설명 |
| --- | --- |
| `-m, --message <MSG>` | 보낼 메시지 텍스트 |
| `--stdin` | 표준 입력에서 메시지 읽기 |
| `-f, --file <PATH>` | 파일에서 메시지 읽기 |

한 번에 하나의 입력 소스만 사용할 수 있습니다.

**예시:**

```bash
# 직접 메시지 보내기
entelecheia-cli chat send -m "안녕하세요, 무엇을 할 수 있나요?"

# 표준 입력에서
echo "src/main.rs의 코드를 분석해 줘" | entelecheia-cli chat send --stdin

# 파일에서
entelecheia-cli chat send -f ./prompts/review.txt
```

`chat send` 명령은 여러 에이전트를 조정하는 핵심 실행 파이프라인인 **스킬 체인**을 통해 메시지를 전달합니다. 실행 중에는 회전 애니메이션으로 진행 상황이 표시됩니다.

### 채팅 기록

```bash
entelecheia-cli chat history [OPTIONS]
```

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `--conversation <ID>` | 대화 ID로 필터링 | — |
| `--agent <TYPE>` | 에이전트 유형으로 필터링 | — |
| `--role <ROLE>` | 역할로 필터링(user/assistant/system) | — |
| `--from <ISO8601>` | 시작 날짜/시간(ISO 8601) | — |
| `--to <ISO8601>` | 종료 날짜/시간(ISO 8601) | — |
| `--limit <N>` | 반환할 최대 메시지 수 | `50` |
| `--offset <N>` | 페이지네이션 오프셋 | `0` |

**예시:**

```bash
entelecheia-cli chat history --agent ApoRia --limit 20 --from 2026-05-01T00:00:00Z
```

### 최근 메시지

```bash
entelecheia-cli chat recent [OPTIONS]
```

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `--timeline <ID>` | 타임라인/세션 ID로 필터링 | — |
| `--agent <TYPE>` | 에이전트 유형으로 필터링 | — |
| `--limit <N>` | 반환할 최대 메시지 수 | `20` |

---

## 에이전트 관리

에이전트 수명 주기 관리(목록, 시작, 중지, 재시작).

```bash
entelecheia-cli agent <COMMAND>
```

### 명령

```bash
# 모든 에이전트 및 상태 나열
entelecheia-cli agent list

# 유형별로 에이전트 시작
entelecheia-cli agent start <AGENT_TYPE>

# 실행 중인 에이전트 중지
entelecheia-cli agent stop <AGENT_TYPE>

# 에이전트 재시작
entelecheia-cli agent restart <AGENT_TYPE>
```

**사용 가능한 에이전트 유형:** ApoRia, EleOs, EpieiKeia, Haplotes, HubRis, Kalos, NeiKos, OreXis, PhiLia, Polemos, SkeMma, SkoPeo.

> 설명: 에이전트는 독립 실행 파일이 아닌 scepter 런타임 내부의 라이브러리 crate으로 실행됩니다. `agent start` 명령은 에이전트 이름과 일치하는 바이너리를 생성하려고 시도하며, 이는 주로 에이전트가 별도 바이너리로 컴파일된 경우에 적용됩니다. 실제 사용에서는 에이전트가 scepter 서버를 통해 활성화됩니다.

---

## 서비스 수명 주기

Docker 컨테이너를 사용하여 Entelecheia(현추) 서비스 스택을 관리합니다.

### 서비스 초기화

```bash
entelecheia-cli init [OPTIONS]
```

전체 서비스 스택 설정: PostgreSQL(pgvector 포함), Docker 레지스트리, scepter 서버 및 WebUI. 필요한 Docker 네트워크를 생성하고 이미지를 가져오거나 빌드합니다.

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `--prefix <STR>` | 컨테이너 이름 프리픽스 | `e-` |
| `--source-build` | 이미지를 가져오는 대신 소스에서 빌드 | `false` |
| `--webui-port <PORT>` | WebUI 포트 | `3424` |

**예시:**

```bash
entelecheia-cli init --prefix ent- --webui-port 8080
```

### 모든 서비스 시작

```bash
entelecheia-cli serve [OPTIONS]
```

이전에 초기화된 모든 컨테이너를 시작합니다. 먼저 `init`을 실행해야 합니다.

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `--prefix <STR>` | 컨테이너 이름 프리픽스 | `e-` |
| `--webui-port <PORT>` | WebUI 포트 | `3424` |

### 모든 서비스 중지

```bash
entelecheia-cli stop [OPTIONS]
```

실행 중인 모든 컨테이너를 순서대로 중지합니다: webui → scepter → registry → postgres.

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `--prefix <STR>` | 컨테이너 이름 프리픽스 | `e-` |

### WebUI만 시작

```bash
entelecheia-cli webui [OPTIONS]
```

WebUI 컨테이너만 시작하거나 생성합니다.

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `--prefix <STR>` | 컨테이너 이름 프리픽스 | `e-` |
| `--webui-port <PORT>` | WebUI 포트 | `3424` |

---

## 구성

시스템 구성을 조회하고 검증합니다.

### 구성 표시

```bash
entelecheia-cli config show
```

다음을 포함한 현재 구성을 표시합니다:

- 데이터베이스 URL 및 연결 설정
- ApoRia LLM 제공자 구성(이름, 모델, 엔드포인트)
- WebSocket 바인딩 주소
- 로그 레벨

API 키는 출력에서 마스킹됩니다(`***`로 표시).

### 구성 검증

```bash
entelecheia-cli config validate
```

검증 검사 수행:

- 데이터베이스 URL 설정 여부
- 완전한 설정을 갖춘 ApoRia 제공자가 하나 이상 구성되었는지 여부
- WebSocket 바인딩 주소 설정 여부

통과/실패 결과를 반환하며, 문제가 있을 경우 세부 정보를 함께 제공합니다.

**출력 예시:**

```text
Validate Configuration:

Validating database configuration...
  [ OK ]  Database URL set

Validating ApoRia LLM configuration...
  [ OK ]  ApoRia providers configured

Validating WebSocket configuration...
  [ OK ]  WebSocket Bind Address set

[ OK ]  Configuration validation passed
```

---

## 연결 컨텍스트

`context` 하위 명령은 명명된 연결 프로필을 관리하여 로컬(Unix socket) 및 원격(WebSocket) scepter 서버 간에 전환할 수 있도록 합니다. 사용 방식은 Docker의 `docker context` 명령과 유사합니다.

### 개념

**컨텍스트**는 CLI가 scepter 서버에 연결하는 방식을 기록한 명명된 구성 파일입니다:

- **local** — Unix socket 연결(기본값, 자동으로 `/run/.../entelecheia-tui.sock`으로 해석)
- **remote** — Bearer token 인증이 포함된 WebSocket 연결

컨텍스트는 `~/.config/entelecheia/contexts/contexts.toml`에 저장됩니다.

### 컨텍스트 나열

```bash
entelecheia-cli context list
```

현재 활성 컨텍스트는 `*`로 표시됩니다.

### 현재 컨텍스트 표시

```bash
entelecheia-cli context show
```

활성 컨텍스트의 유형, socket 경로, WS URL 및 설명 정보를 표시합니다.

### 컨텍스트 생성

```bash
# 원격 WebSocket 컨텍스트
entelecheia-cli context create staging \
  --ws-url ws://scepter.example.com:8424/ws \
  --bearer-token <TOKEN> \
  --description "Staging server"

# 추가 로컬 컨텍스트
entelecheia-cli context create dev --description "Development server"
```

원격 서버에서 Bearer token 가져오기:

```bash
# 서버 머신에서
docker exec e-scepter cat /home/entelecheia/.config/entelecheia/scepter.token
```

### 컨텍스트 전환

```bash
entelecheia-cli context use staging
# 이후 모든 명령(send, status, chat 등)은 staging 연결을 통해 라우팅됩니다
```

### 컨텍스트 제거

```bash
entelecheia-cli context remove staging
```

`default` 컨텍스트는 제거할 수 없습니다.

### 예시 워크플로

```bash
# 현재 컨텍스트 보기
entelecheia-cli context list

# 스테이징 서버용 원격 컨텍스트 생성
entelecheia-cli context create staging \
  --ws-url ws://192.168.1.100:8424/ws \
  --bearer-token $(cat /path/to/token)

# 스테이징 환경으로 전환
entelecheia-cli context use staging

# 원격 서버를 통해 메시지 보내기
entelecheia-cli send "현재 할 일 목록 표시"

# 원격 서버 상태 확인
entelecheia-cli status

# 로컬로 다시 전환
entelecheia-cli context use default
```

---

## 상태 및 모니터링

### 시스템 상태

```bash
entelecheia-cli status
```

다음을 표시합니다:

- 서버 버전
- 연결 상태(socket 상태)
- LLM 제공자 요약
- WebSocket 바인딩 주소
- 에이전트 목록 및 실행/중지 상태
- 시스템 자원(메모리 사용량, 평균 부하)

### 상태 경로 조회

`status` 명령은 경로 유사 매개변수를 받아 특정 하위 시스템을 조회합니다. 구문은 agent 범위의 타임라인, 채팅 기록 검사 및 장치 열거를 지원합니다.

```bash
entelecheia-cli status <PATH> [--raw]
```

| 경로 구문 | 설명 |
| --- | --- |
| `timeline.#agent[-N]` | 특정 agent의 최근 N회 스킬 호출 기록 표시 |
| `timeline.#agent[N][M]` | N번째 스킬 호출의 M번째 MCP/도구 호출 표시 |
| `history[-N]` | 최근 N개의 채팅 메시지 표시(모든 역할) |
| `history[-N].body` | 뒤에서 N번째 메시지의 본문 표시 |
| `device` | Polemos가 식별한 모든 엣지 장치 나열 |
| `device[N]` | N번째 Polemos 장치의 세부 정보 표시 |

**예시:**

```bash
# Haplotes #001 agent의 최근 30회 스킬 스케줄링 기록
entelecheia-cli status timeline.#hap_lotes.001[-30]

# 3번째 스킬 호출의 2번째 MCP/도구 호출
entelecheia-cli status timeline.#hap_lotes.001[3][2]

# 최근 30개 메시지
entelecheia-cli status history[-30]

# 뒤에서 3번째 메시지 본문(일반 텍스트)
entelecheia-cli status history[-3].body --raw

# 모든 Polemos 장치
entelecheia-cli status device

# 3번째 Polemos 장치 세부 정보
entelecheia-cli status device[3]
```

> **Shell 주의:** bash/zsh에서는 glob 확장을 방지하기 위해 `[...]`가 포함된 경로를 작은따옴표로 묶으십시오: `entelecheia-cli status 'history[-30]'`. `#` 문자가 단어 중간에 포함된 경우 이스케이프할 필요가 없습니다. fish shell에서는 위 경로 모두 따옴표가 필요하지 않습니다.

상태 경로 조회는 Unix socket JSON-RPC를 통해 서버와 통신합니다. `timeline.*` 및 `history.*` 조회는 서버가 실행 중이어야 합니다. `device` 조회는 서버에 Polemos 작업 영역 등록이 필요합니다.

### 로그 보기

```bash
entelecheia-cli logs [OPTIONS]
```

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `-a, --agent <NAME>` | 에이전트 이름으로 로그 필터링 | 모든 에이전트 |
| `-l, --lines <N>` | 표시할 줄 수(꼬리) | `100` |

**예시:**

```bash
# 모든 에이전트 로그의 마지막 200줄 표시
entelecheia-cli logs -l 200

# ApoRia 로그 표시
entelecheia-cli logs -a ApoRia
```

로그는 `./logs/` 디렉터리에서 읽습니다. 각 에이전트에는 개별 로그 파일(`ApoRia.log`, `EleOs.log` 등)이 있습니다.

---

## 구독 (Layer3)

Layer3 에이전트 구독 관리 — 설치 및 실행 가능한 외부 에이전트 패키지.

### 구독 나열

```bash
entelecheia-cli subscribe list
```

상태(설치됨/대기 중), 활성화 상태, 자동 업데이트 설정 및 소스를 포함한 모든 구성된 구독을 표시합니다.

### 구독 추가

```bash
entelecheia-cli subscribe add [OPTIONS]
```

| 옵션 | 설명 |
| --- | --- |
| `--name <NAME>` | 구독 이름(필수) |
| `--source <SOURCE>` | 소스 유형: `official`, `github` 또는 `url`(필수) |
| `--repository <REPO>` | GitHub 저장소(github 소스용) |
| `--url <URL>` | 직접 URL(url 소스용) |
| `--version <VER>` | 버전 제약 |
| `--auto-update` | 자동 업데이트 활성화 |
| `--disabled` | 비활성화 상태로 추가 |

**예시:**

```bash
entelecheia-cli subscribe add --name my-agent --source github --repository user/repo
```

### 구독 제거

```bash
entelecheia-cli subscribe remove <NAME>
```

### 구독 동기화

```bash
# 모든 구독 동기화
entelecheia-cli subscribe sync

# 특정 구독 동기화
entelecheia-cli subscribe sync --name my-agent
```

### 자동 업데이트

```bash
entelecheia-cli subscribe auto-update
```

`auto_update`가 활성화된 모든 구독을 업데이트합니다.

---

## 에이전트 실행

```bash
entelecheia-cli run <AGENT> [OPTIONS]
```

Layer3 에이전트 스크립트를 실행합니다. 현재 디렉터리에서 `.amphoreus/<AGENT>/run.py`를 찾습니다. 최초 실행 시 사전 검사 감사를 실행합니다.

| 옵션 | 설명 |
| --- | --- |
| `--ci` | CI 모드 활성화 |
| `--auto-pr` | 자동 PR 모드 활성화 |
| `--dry-run` | 시험 실행(실제 변경 없음) |
| `--providers <LIST>` | 쉼표로 구분된 제공자 목록 |
| `--output-dir <DIR>` | 출력 디렉터리 |

**예시:**

```bash
# 시험 실행 모드로 Layer3 에이전트 실행
entelecheia-cli run my-agent --dry-run

# 지정된 제공자로 실행
entelecheia-cli run my-agent --providers openai,anthropic

# CI 모드 및 자동 PR 제출
entelecheia-cli run my-agent --ci --auto-pr

# 백그라운드 모드로 실행(즉시 반환, 자식 프로세스가 백그라운드에서 실행)
entelecheia-cli -d run my-agent --ci --auto-pr
```

### 백그라운드 모드(`-d` / `--daemon`)

백그라운드 모드 플래그는 CLI가 `--daemon` 매개변수를 제거한 상태로 분리된 자식 프로세스를 다시 생성하고 즉시 반환하도록 합니다. 자식 프로세스는 원래 명령을 상속받아 독립적으로 실행됩니다. 이후 `status`를 사용하여 진행 상황을 확인할 수 있습니다.

`run`, `init`, `deploy` 등 장시간 실행 작업에 적합합니다:

```bash
# 백그라운드로 agent 실행 디스패치
entelecheia-cli -d run my-agent

# 백그라운드로 서비스 초기화 디스패치
entelecheia-cli -d init --prefix prod-

# 나중에 상태 확인
entelecheia-cli status
entelecheia-cli status history[-5]
```

---

## 타임라인

세션 타임라인을 조회합니다.

### 타임라인 나열

```bash
entelecheia-cli timeline list [OPTIONS]
```

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `--agent <TYPE>` | 에이전트 유형으로 필터링 | — |
| `--limit <N>` | 최대 결과 수 | `50` |
| `--offset <N>` | 페이지네이션 오프셋 | `0` |

### 타임라인 세부 정보 표시

```bash
entelecheia-cli timeline show <CONVERSATION_ID> [OPTIONS]
```

| 옵션 | 설명 | 기본값 |
| --- | --- | --- |
| `--include-messages` | 출력에 메시지 포함 | `true` |

---

## Docker 이미지

```bash
entelecheia-cli init-docker-images [OPTIONS]
```

플랫폼에 필요한 Docker 이미지를 빌드하거나 가져옵니다.

| 옵션 | 설명 |
| --- | --- |
| `--source-build` | 이미지를 가져오는 대신 소스에서 빌드 |
| `--tag <TAG>` | 이미지 태그(기본값: `latest`) |

**예시:**

```bash
# 소스에서 모든 이미지 빌드
entelecheia-cli init-docker-images --source-build

# 사용자 정의 태그로 가져오기
entelecheia-cli init-docker-images --tag v0.2.0
```

관리되는 이미지:

- `entelecheia` — 오케스트레이션 서버(내장 cosmos 런타임 포함)
- `pgvector/pgvector` — 벡터 확장이 포함된 PostgreSQL

---

## 고급 사용법

### 스크립트용 JSON 출력

`--format json`을 사용하여 기계 판독 가능한 출력을 얻고, `jq` 또는 다른 도구로 파이프할 수 있습니다:

```bash
entelecheia-cli status --format json | jq '.server_version'
entelecheia-cli chat history --format json | jq '.messages[].content'
```

### 연쇄 정리 및 초기화

```bash
# 완전히 해체하고 재구축
entelecheia-cli --clean && entelecheia-cli init --prefix my-
```

### 디버그 모드

```bash
# 디버깅을 위해 trace 레벨 로그 활성화
entelecheia-cli -l trace send "테스트 메시지"
```

### TUI와 함께 사용

CLI와 TUI는 동일한 scepter 서버에 연결됩니다. 둘 다 동시에 사용할 수 있습니다:

- 대화형 세션을 위해 TUI 시작: `cargo run --bin entelecheia-tui`
- 스크립트 작성, 자동화 및 빠른 조회를 위해 CLI 사용

---

## 문제 해결

### "No command specified"

`--help`를 실행하여 사용 가능한 명령을 확인하거나 `send "메시지"`로 빠르게 메시지를 보내십시오.

### "Failed to connect to Docker"

Docker(또는 Podman)가 실행 중인지 확인하십시오:

```bash
docker info
docker run hello-world
```

### "Agent binary not found"

에이전트는 독립 바이너리가 아닌 scepter 런타임의 내부 라이브러리 crate입니다. scepter 서버를 시작하여 에이전트를 활성화하십시오:

```bash
entelecheia-cli init && entelecheia-cli serve
```

### "No LLM providers configured"

환경 변수를 통해 ApoRia 제공자 구성을 설정하십시오. 제공자 설정 설명은 [빌드 가이드](building.md)를 참조하십시오.

### "Configuration validation failed"

`entelecheia-cli config validate`를 실행하여 어떤 검사가 실패했는지 확인하십시오. 일반적인 문제:

- `DATABASE_URL` 환경 변수 누락
- ApoRia 제공자 설정 불완전(이름, 모델, `api_key`)
- WebSocket 바인딩 주소 누락
