+++
title = "오류 코드 참조"
description = """> 중요: 아래 문서화된 "오류 코드"(예: `DB_CONNECT_FAILED`, """
lang = "ko"
category = "design"
subcategory = "core"
+++

# 오류 코드 참조

> **중요**: 아래 문서화된 "오류 코드"(예: `DB_CONNECT_FAILED`,
> `LLM_CALL_FAILED`)는 소스 코드에서 추출된 **문자열 기반 메시지 패턴**으로,
> 비공식적인 편의 레이블이며 공식적인 오류 분류 체계가 아닙니다.
> **권위 있는 구조화된 오류 유형**은
> [`packages/shared/core/src/errors.rs`](../packages/shared/core/src/errors.rs)에
> 정의된 열거형들입니다: `AgentErrorCode`(8행), `StructuredAgentError`, `CoreError`,
> `CredentialError`, `PromptLoadError`, `SoulLoadError` 및 그 변형들.
> 프로그래밍 방식으로 오류를 통합하거나 보고할 때는 여기에 나열된 문자열 패턴이 아닌
> 이러한 열거형을 사용하십시오.

본 문서는 Entelecheia의 Rust 코드베이스 전반에서 사용되는 오류 패턴을 목록화합니다.
구조화된 오류 코드는 작업 진행 중이며, 현재 대부분의 오류는 설명적 메시지와 함께
`anyhow`/`thiserror`를 사용합니다.

## 오류 범주

### 데이터베이스 (`DB_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `DB_CONNECT_FAILED` | `데이터베이스 연결 실패: {}` | `scepter/src/app/setup.rs` |
| `DB_MIGRATE_FAILED` | `데이터베이스 마이그레이션 실패: {}` | `scepter/src/app/setup.rs` |
| `DB_TABLE_CHECK_FAILED` | `테이블 존재 확인 실패: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_INIT_SCRIPT_FAILED` | `초기화 스크립트 실행 실패: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_SAVE_FAILED` | `에이전트 정보 저장 실패: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_UPDATE_FAILED` | `에이전트 상태 갱신 실패: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_LOG_FAILED` | `로그 항목 기록 실패: {}` | `packages/shared/infra_services/src/persistence.rs` |
| `DB_CLEANUP_FAILED` | `오래된 로그 정리 실패: {}` | `packages/shared/infra_services/src/persistence.rs` |

### 구성 (`CFG_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `CFG_CREDENTIAL_INIT_FAILED` | `자격 증명 저장소 초기화 실패: {}` | `scepter/src/app/setup.rs` |
| `CFG_PROVIDER_INIT_FAILED` | `제공자 구성 초기화 실패: {}` | `scepter/src/app/setup.rs` |
| `CFG_MODEL_INIT_FAILED` | `모델 구성 초기화 실패: {}` | `scepter/src/app/setup.rs` |
| `CFG_USER_INIT_FAILED` | `사용자 구성 초기화 실패: {}` | `scepter/src/app/setup.rs` |
| `CFG_KEY_STORE_INIT_FAILED` | `키 저장소 서비스 초기화 실패: {}` | `scepter/src/app/setup.rs` |

### 상태 (`ST_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `ST_SERIALIZE_FAILED` | `상태 직렬화 실패: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_WRITE_FAILED` | `임시 파일 쓰기 실패: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_READ_FAILED` | `상태 파일 읽기 실패: {}` | `scepter/src/state/state_persistence.rs` |
| `ST_PARSE_FAILED` | `상태 파일 파싱 실패: {}` | `scepter/src/state/state_persistence.rs` |

### WebSocket (`WS_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `WS_SEND_FAILED` | `메시지 전송 실패: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_TIMEOUT` | `응답 대기 시간 초과 또는 채널 닫힘` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_PARSE_FAILED` | `에이전트 목록 파싱 실패: {}` | `packages/shared/infra_services/src/ws_transport.rs` |
| `WS_NOT_CONNECTED` | `websocket 연결이 설정되지 않음` | `packages/shared/infra_services/src/ws_transport.rs` |

### 에이전트 (`AG_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `AG_CONNECT_FAILED` | `연결 실패: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_SEND_FAILED` | `전송 실패: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_CHANNEL_NOT_INIT` | `전송 채널이 초기화되지 않음` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_REGISTRATION_FAILED` | `등록 메시지 전송 실패: {}` | `packages/shared/core/src/errors.rs:7-28` |
| `AG_RE_REGISTER_FAILED` | `내부 에이전트 재등록 실패` | `scepter/src/state/state_restoration.rs` |

### LLM (`LLM_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `LLM_CALL_FAILED` | `LLM 호출 실패: {}` | `scepter/src/state_machine/llm_chat/chat_loop.rs` |

### Layer2 에이전트 (`L2_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `L2_INIT_FAILED` | `layer2 에이전트 구성 초기화 실패: {}` | `scepter/src/app/setup.rs` |
| `L2_SKILLS_VALIDATE_FAILED` | `layer2 에이전트 스킬 검증 실패: {}` | `scepter/src/app/setup.rs` |

### 스킬 (`SK_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `SK_PROMPT_LOAD_FAILED` | `프롬프트 로더 오류` | `packages/shared/prompt/src/prompt_loader.rs` |
| `SK_TOML_PARSE_FAILED` | `TOML 파싱 실패: {}` | `packages/shared/prompt/src/prompt_loader.rs` |

### 런타임 (`RT_*`)

| 코드 | 메시지 패턴 | 출처 |
| --- | --- | --- |
| `RT_ARC_UNWRAP_DOMAIN` | `Arc::try_unwrap이 llm_domain/agent_domain에 대해 실패` | `scepter/src/state_machine/mod.rs` |
| `RT_UNDO_NO_ACTIVE_SKILL` | `active_streaming_skill이 None, HubRis로 기본값 설정` | `scepter/src/state_machine/mod.rs` |

---

> **참고**: 이는 최선의 노력을 다한 목록입니다. Entelecheia는 고유 코드를
> 가진 구조화된 오류 유형으로 마이그레이션 중입니다. 이 참조를 확장하기 위한
> 기여를 환영합니다.
