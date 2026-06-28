+++
title = "산업 회랑 통합 계획"
description = """> 목표: 시스템은 완전히 알려지지 않은 산업 실증 회랑에 대해 자율적 자가 인터페이스를 시연해야 합니다"""
lang = "ko"
category = "architecture"
subcategory = "core"
+++

# 산업 회랑 자율 통합 계획

> **목표**: 시스템은 완전히 알려지지 않은 산업 실증 회랑에 대해 **자율적 자가 인터페이스**를 시연해야 합니다 — 하드웨어 검색, 데이터 모델 추론, 모니터링 구성 생성, 경보→응답 루프 폐쇄를 수동 장치별 엔지니어링 없이 수행합니다.
> **정부 엄격한 기한**: 이 기능은 정부 프로젝트 마일스톤에 묶여 있습니다.

---

## 남은 작업

전체 검색 → 추론 → 모니터링 → 경보 → **쓰기 승인** 체인이 출시되었습니다(Phase A.1–A.3, B, C, D.1, **D.2 ✓**). 유일한 남은 작업은 **종단 간 도그푸드 검증(Phase E)** — 운영적, 코드 아님.

### D.2 — 쓰기 승인 왕복 (인간 루프 내) ✓

```text
에이전트가 쓰기가 필요하다고 결정
  → verify_write_safety → 거부됨
    → orexis.request_write_approval → WriteApprovalRequest 브로드캐스트
      → shittim-chest가 승인 대화상자 표시 (industrial.approveWrite)
        → [승인됨] → 임시 화이트리스트 항목 → 실행 + 재읽기 검증
        → [거부됨]   → 에이전트가 거부 수신, 계획 조정
```

**구현됨:**

| # | 작업 | 파일 | 상태 |
| --- | --- | --- | --- |
| A.2.4.1 | `orexis.request_write_approval` MCP 도구 — `WriteApprovalRequest` 빌드, `TuiMessage::IndustrialWriteApprovalPush` 브로드캐스트, 운영자 응답까지 일시 중단(oneshot + timeout) | `packages/agents/orexis/src/mcp/tools/industrial_write_tools.rs` | ✓ |
| A.2.4.2 | `industrial.approveWrite` WS 핸들러 — 공유 `WriteApprovalRegistry`를 통해 대기 중 요청 해결; 승인 시 후속 쓰기가 `verify_write_safety`를 통과하도록 임시 화이트리스트 항목 추가 | `packages/scepter/src/tui_connection/mod.rs` | ✓ |

생산자/해결자는 프로세스 전반의 공유 `WriteApprovalRegistry`(`_shared_security_policy::write_approval_registry`)를 통해 분리되며, orexis 시작 시 주입되고 운영자 응답 시 scepter에서 사용됩니다.

---

## Phase E: 종단 간 도그푸드

운영 검증, 순수 코드 아님. 하드웨어 시뮬레이터 실행 필요.

### E.1 — 테스트 환경

| # | 구성 요소 | 설정 |
| --- | --- | --- |
| E.1.1 | S7comm 시뮬레이터 | 가상 S7-1500으로 `snap7-server` 크레이트 실행. DB1에 사전 로드: 오프셋 0에 REAL 온도, 오프셋 4에 REAL 압력, 오프셋 8에 INT 유량, 오프셋 10에 BOOL 밸브, 그리고 50바이트 무작위 데이터 |
| E.1.2 | Modbus 시뮬레이터 | 가상 직렬 포트에서 aoba 슬레이브 모드 실행(`socat pty pty`). 스테이션 5에 알려진 레지스터 값 사전 로드 |
| E.1.3 | Entelecheia + evernight | 표준 docker-compose 시작. evernight `sensor-poll`이 `--manifest` 플래그로 준비됨 |

### E.2 — 도그푸드 시나리오

| # | 시나리오 | 단계 | 통과 기준 |
| --- | --- | --- | --- |
| E.2.1 | **알려지지 않은 S7comm 회랑** | (1) 시스템에 대상 `192.168.1.10:102` 제공. (2) `industrial_discover` 스킬 체인이 자율 실행. (3) 시스템이 S7comm 프로토콜, DB1 검색, 필드 의미 추론, 매니페스트 생성. (4) 운영자가 TUI에서 매니페스트 검토. (5) 승인 → evernight가 폴링 시작. (6) 경보 값 주입 → Hubris alarm_response 트리거 → 시정 조치 제안. | 매니페스트가 ≥ 3개 올바르게 추론된 필드로 생성됨. 경보가 `alarm_response → task_decompose → plan_execute` 체인 트리거. |
| E.2.2 | **알려지지 않은 Modbus 회랑** | 가상 직렬 포트에서 Modbus RTU로 동일 흐름. 다른 스테이션 레이아웃. | 동일 기준. |
| E.2.3 | **혼합 프로토콜 검색** | 두 시뮬레이터 동시 실행. 시스템이 둘 다 검색, 결합 매니페스트 생성. | 두 스테이션이 올바른 프로토콜로 매니페스트에 나타남. |
| E.2.4 | **쓰기 승인 흐름** | 에이전트가 밸브 닫기 제안(검색된 BOOL 필드에 쓰기). `verify_write_safety` 차단(화이트리스트 없음). WriteApprovalRequest가 운영자에게 전송. 운영자 승인. 재읽기 검증과 함께 쓰기 실행. | 전체 왕복: 제안 → 차단 → 요청 → 승인 → 실행 → 검증. **(D.2 출시됨 — 도그푸드 준비.)** |

### E.3 — 데모 녹화

| # | 작업 | 비고 |
| --- | --- | --- |
| E.3.1 | 전체 검색→모니터링→경보→응답 주기를 화면 캡처로 녹화 | 알려지지 않은 하드웨어에 대한 자율 적응 시연 |
| E.3.2 | 검색 보고서 아티팩트 생성(자동 생성 매니페스트 TOML + 추론된 필드 테이블) | 정부 마일스톤 검토를 위한 유형적 결과물 |

---

## 형제 프로젝트 의존성 (남은)

| 형제 | 우리가 필요한 것 | 시기 | 상태 |
| --- | --- | --- | --- |
| **arona** | `WriteApprovalRequest`에 대한 WS 브로드캐스트 경로 (A.2.4) | ~~A.2.4 / D.2 차단~~ 완료 — `TuiMessage::IndustrialWriteApprovalPush`(arona 타입에서 재내보내기)를 통해 전달 | ✓ |
| **shittim-chest** | 운영자 승인 대화상자(`industrial.approveWrite` 소비자) + 검색 진행 상황 렌더링 | E.2.4 도그푸드 차단(scepter의 WS 핸들러 준비됨; shittim-chest가 대화상자 렌더링 및 응답 POST 필요) | 형제 PLAN |

---

## 명시적 범위 외 (2주 스프린트)

- OPC UA 클라이언트/서버 (Rust 생태계 준비 안 됨)
- EtherNet/IP / CIP (Rockwell)
- EtherCAT (Beckhoff)
- CAN 버스
- 프론트엔드 테스트 커버리지 (shittim-chest는 가이드 계획만, 테스트 작성 없음)
- CLI 기능 동등성 (TUI 수준)

---

# 기술 로드맵 — 아키텍처 심화

> **날짜**: 2026-06-26
> **컨텍스트**: 700개 이상의 오래된 문서/파일 정리 및 모든 프롬프트를 `res/prompts/`로 통합한 후, 실제 소스 코드 대비 나머지 설계 문서를 감사하여 구현할 가치가 있는 포부적 설계를 식별했습니다.

---

## 1. 하위 배지 주소 지정 + 병렬 스킬 실행

**판정**: 구현할 가치 있음. 인프라 ~80% 구축, 마지막 20%만 누락.

**현재 상태**:

- `BadgeRegistry` (`packages/scepter/src/state_machine/badge_registry.rs:92-120`)가 이미 부모-자식 `link_sessions()` 지원.
- `#001.005` 하위 배지 구문 파싱이 `find_by_container_id_or_sub()`에 존재하나 별개의 자식 컨테이너로 해결하는 대신 하위 번호 제거.
- `SnowflakeContainer.parent_id` 및 `branch_level` 필드가 존재하나 메타데이터 전용 — 라우팅에 사용된 적 없음.
- 엣지 노드 우선순위 큐잉(`edge_node_registry.rs:73-126`)이 세분화된 리소스 잠금 준비 완료.
- 스킬 체인이 엄격히 **직렬** — `pipeline.rs:68-226`이 한 번에 하나의 스킬만 루프. 독립적인 `next_targets`가 있는 조정자 스킬이 병렬 실행 가능한 상황에서 직렬 실행.

**누락된 것**:

1. ✅ `find_by_container_id_or_sub()`가 `#001.005` → 부모 컨테이너의 가장 깊은 활성 포크된 자식으로 해결, 포크가 없으면 부모로 폴백(하위 호환).
1. ✅ `SnowflakeManager`에 자식/자손 조회 추가: `children_of`, `children_of_badge`, `most_recent_child_of`, `deepest_descendant` (`parent_id` → 역방향 인덱스).
1. ✅ `next_targets`의 `FuturesUnordered` 기반 병렬 실행: `dispatch_parallel_targets`가 조정자의 독립적인 **리프** 대상을 `parallel_dispatch::fan_out`을 통해 동시에 팬아웃(`Semaphore`로 제한). 직렬 `invoke_skill_with_retries` 경로의 두 전역 싱글톤 차단 요소 처리:

   - **공유 로컬 cosmos 네임스페이스** → Phase 1에서 각 대상이 **자체 cosmos 컨테이너**로 포크됨(`fork_container_for_skill` + `assign_container_id` + `register_container_badge_in_registry`), 따라서 `dump/restore_cosmos_namespace`는 브랜치별 no-op이며 동시 실행은 격리됨. `MAX_BRANCH_DEPTH`(항목 4)가 포크 체인 제한.
   - **`active_streaming_skill` UI 경쟁** → 허용(Option에 대한 마지막 쓰기 승리; 각 브랜치 후 None으로 재설정).
   - **`&mut SkillChainInput` 스레딩** → `BranchOwner`가 브랜치별 가변 부분 미러링; `as_input`이 단명 `SkillChainInput`으로 차용하여 변경되지 않은 파이프라인 헬퍼 재사용.

Phase 1(포크 + 준비 + 프롬프트 빌드 + 도구 화이트리스트)은 `rag_buffer` 경쟁을 피하기 위해 **직렬화**됨; Phase 2(지연 시간이 큰 LLM 호출)만 병렬 실행; Phase 3은 정리 및 보고서를 부모 컨텍스트로 병합(`merge_branch_reports`). `SKILL_CHAIN_PARALLEL_TARGETS`(기본값 **꺼짐**) + `parallel_targets_eligible`(컨테이너화 + 모든 리프 대상) 뒤에 게이트됨. `route_to_next_skill`의 직렬 스택 언와인드가 기본값 유지.

1. ✅ 두 포크 경로에서 `MAX_BRANCH_DEPTH`(`COSMOS_MAX_BRANCH_DEPTH`, 기본값 4) 집행; 자식이 하드코딩된 `1` 대신 `source.branch_level + 1`에 등록.

**예상 효과**: `industrial_discover`와 같은 조정자 스킬의 병렬 파일 쓰기, 병렬 분석이 종단 간 지연 시간을 크게 감소시킬 것입니다.

---

## 2. 메모리 침적 파이프라인

**판정**: 품질 승수, 중요하지 않음. 장기 로드맵용으로 예약.

**현재 상태**:

- `PhiliaMemoryService`는 대사 작용이 없는 평평한 "저장 → 임베딩 → 검색" 그래프.
- `memory_consolidate`는 사소함 — 에피소드 노드 생성만, 추상화/요약 없음.
- 메모리 붕괴, 노화, 진부화 점수, 노드 간 품질 그라데이션 없음.
- 모든 노드가 미분화 `MemoryNode` — 에피소드/절차/원자 분리 없음.
- 인메모리 벡터 검색이 O(n) 무차별 대입(장기적으로 확장 불가).
- `KnowledgeStore`(별도 시스템)는 생명주기 단계(Created→Vectorized→Searchable→Consolidated→Deprecated)와 합의 검증 보유 — 침적에 가장 가까운 기존 유사체.

**시급하지 않은 이유**:

- RAG 컨텍스트 주입(`RagContextBuffer` → LLM 쿼리 재작성 → `bundle_search`)이 현재 도구 호출 에이전트에 충분한 컨텍스트 제공.
- pgvector HNSW 인덱스가 프로덕션 규모 검색 처리.
- 시스템이 "저장 및 검색"으로 작동 — 침적은 "대사 작용"으로 만들겠지만, 이는 기능적 공백이 아닌 점진적 품질.

**향후 작업** (일정 없음):

- 자동 통합: 주기적 LM 구동 관련 노드 요약을 상위 "에피소드"로.
- 품질 그라데이션: 접근 횟수, 시간적 붕괴, 신뢰도 점수.
- 차별화된 검색 전략이 있는 3채널 프로토타입(에피소드/절차/원자).

---

## 3. 에이전트 간 협상

**판정**: 낮은 우선순위. 기본 요소가 저수준 빌딩 블록으로 존재; 즉각적인 사용 사례 없음.

**현재 상태**:

- `deliver_message(message_type="Question")` 존재(`epieikeia/src/mcp/tools/deliver_message.rs:63`) — 다른 에이전트의 메일함에 질문 푸시 가능.
- `inject_user_prompt` / `consume_injected_prompts` 존재하나 **폴 기반** — 파이프라인 통합 없음. 에이전트가 명시적으로 `consume_injected_prompts`를 호출하여 메일 확인 필요.
- `Haplotes`가 `AskAgent` / `ReplyAgent` / `Escalated` 대화 라우팅 타입 보유 — 그러나 모두 비즈니스 로직이 없는 no-op ACK.
- `NEGOTIATION_ROUND_TIMEOUT_SECS` / `NEGOTIATION_TOTAL_TIMEOUT_SECS` 환경 변수가 `RuntimeTuningConfig`에 정의됨, **그러나 어디서도 소비되지 않음** — 데드 코드.

**낮은 우선순위인 이유**:

- 현재 순차적 스킬 체인 디스패치 + 문자열로의 컨텍스트 전달이 모든 현재 사용 사례 처리.
- 병합 충돌은 단일 스킬 디스패치(`resolve_merge_conflict`)로 처리되며, 이는 충분.
- 협상 루프(스킬 체인 인터셉트 → 에이전트에 질문 → 응답 대기 → 통합)는 구축 및 테스트가 복잡할 것. 아직 프로덕션 사용 사례가 요구하지 않음.

**재검토 시기**: 에이전트가 언젠가 동적으로 중간 체인 결정을 협상해야 하는 경우(디스패치 후 대기뿐 아니라), 기본 요소는 40% 구축됨. 공백은 파이프라인 통합 루프.

---

## 요약

| 기능 | 인프라 구축 | 우선순위 | 다음 단계 |
| --- | --- | --- | --- |
| 하위 배지 + 병렬 실행 | 100% | **높음** | ✅ 완료 — 하위 배지→자식, 자식 인덱스, 브랜치 깊이 및 루프 내 병렬 디스패치 모두 출시(병렬 기본 꺼짐) |
| 메모리 침적 | 20% | **장기** | 즉시 조치 없음; 병렬 실행 후 재검토 |
| 에이전트 간 협상 | 40% | **낮음** | 구체적 사용 사례 대기; 기본 요소 준비됨 |
