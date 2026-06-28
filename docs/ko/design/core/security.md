+++
title = "Entelecheia 보안 아키텍처"
description = """> Entelecheia 다중 에이전트 오케스트레이션 플랫폼을 위한 포괄적인 심층 방어 모델."""
lang = "ko"
category = "design"
subcategory = "core"
+++

# Entelecheia 보안 아키텍처

> Entelecheia 다중 에이전트 오케스트레이션 플랫폼을 위한 포괄적인 심층 방어 모델.

## 개요

Entelecheia는 하드웨어 수준 컨테이너 격리부터 LLM 대면 도구 권한 게이트까지 **14개의 독립적으로 테스트 가능한 보안 계층**에 걸친 **심층 방어 보안 아키텍처**를 구현합니다. 모든 도구를 LLM에 직접 노출하는 기존 에이전트 프레임워크와 달리, Entelecheia의 **Exec-Only 마이크로커널** 설계는 LLM이 3개의 프리미티브 도구(`exec`, `write_to_var`, `write_to_var_json`)만 볼 수 있으며, 148개의 MCP 도구는 다중 계층 인가를 거친 타입화된 IEPL 파이프라인을 통해 디스패치됩니다.

## 보안 계층 색인

| # | 계층 | 크레이트 | 완화되는 위협 |
| --- | --- | --- | --- |
| 1 | Exec-Only 마이크로커널 | `scepter`, `mcp_types` | LLM의 무제한 도구 접근 |
| 2 | 이중 인가 권한 게이트 | `security_policy` | 인가되지 않은 MCP 도구 호출 |
| 3 | 신뢰 수준 스킬 인가 | `domain_skills_permissions` | 스킬 체인을 통한 권한 상승 |
| 4 | 컨테이너 격리 (외부) | `container` (Docker/Podman) | 에이전트 코드로 인한 호스트 침해 |
| 5 | OCI 샌드박스 (내부) | `container_runtime` (Youki/libcontainer) | 컨테이너 탈출 |
| 6 | RBAC 접근 제어 | `domain_auth`, shittim-chest `rbac` | 인가되지 않은 API 접근 |
| 7 | JWT 인증 | shittim-chest `auth` (HS256) | 세션 하이재킹, 재생 공격 |
| 8 | API 키 암호화 | `aporia` (AES-256-GCM) | 저장된 크리덴셜 유출 |
| 9 | 보안 센티넬 | `orexis` (OreXis 에이전트) | 악성 코드 실행, 규정 준수 위반 |
| 10 | IEPL 타입 안전 파이프라인 | `iepl`, `iepl_engine`, `skemma` | 비타입화 도구 호출을 통한 주입 |
| 11 | 프로바이더 레지스트리 화이트리스트 | `config/registries.toml` | 신뢰할 수 없는 패키지로 인한 공급망 공격 |
| 12 | 프롬프트 주입 방어 | IEPL 샌드박스 경계 | 도구 출력을 통한 LLM 프롬프트 주입 |
| 13 | 속도 제한 | shittim-chest `channel/rate_limit` | DoS, 리소스 고갈 |
| 14 | 감사 추적 | `orexis`, `timeline` | 사후 포렌식, 책임 추적성 |

---

## 계층 1: Exec-Only 마이크로커널

**크레이트:** `scepter`, `mcp_types`
**설계 철학:** LLM 공격 표면 최소화

LLM은 세 가지 프리미티브 연산만 호출할 수 있는 **exec-only 샌드박스**에서 작동합니다:

| 도구 | 목적 | 매개변수 |
| --- | --- | --- |
| `exec` | 스크립트 문자열 실행 | JavaScript 코드 (IEPL을 통해 TypeScript에서 트랜스파일됨) |
| `write_to_var` | 문자열 값 저장 | 변수명 + 값 |
| `write_to_var_json` | JSON 값 저장 | 변수명 + JSON 값 |

148개의 모든 MCP 도구(파일 연산, 컨테이너 관리, 장치 제어, 웹 검색 등)는 **LLM에게 보이지 않습니다**. 이들은 LLM의 `exec`가 ES 모듈 임포트(예: `import { file_read } from 'kalos'`)를 호출할 때 IEPL 파이프라인을 통해 간접적으로 호출됩니다.

**위협 모델:** LLM이 프롬프트 주입을 통해 침해되더라도 `container_destroy`나 `ssh_exec`와 같은 위험한 도구를 직접 호출할 수 없습니다. IEPL 파이프라인은 모든 도구 실행 전에 타입 검사와 권한 확인을 강제합니다.

**구현:** `packages/shared/mcp_types/src/`가 마이크로커널 IPC 타입을 정의합니다. `packages/cosmos/`의 `exec` 핸들러는 스크립트를 트랜스파일하고 Boa 엔진을 통해 실행하며, `skemma`의 `McpRouter`를 통해 도구 호출을 라우팅합니다.

---

## 계층 2: 이중 인가 권한 게이트

**크레이트:** `security_policy` (5,772 라인)

모든 MCP 도구는 **권한 수준** 열거형을 통해 접근 요구 사항을 선언합니다. 모든 스킬(IEPL 스크립트)은 도구별로 필요한 권한 수준을 선언합니다. 호출이 진행되려면 양쪽이 모두 동의해야 합니다.

```rust
pub enum PermissionLevel {
    /// 읽기 전용 연산 (file_read, list_dir 등)
    Read,
    /// 워크스페이스 내 쓰기 연산 (file_write, exec_script)
    Write,
    /// 외부 시스템에 영향을 미치는 연산 (ssh_exec, container_deploy)
    System,
    /// 되돌릴 수 없는 결과를 초래하는 연산 (container_destroy, device_reboot)
    Destructive,
}
```

**인가 흐름:**

1. 스킬 선언: "나는 `ssh_exec`에 `System` 접근이 필요합니다"
1. 도구 선언: "나는 `System` 권한이 필요합니다"
1. 권한 게이트 확인: `skill_level >= tool_requirement` AND `스킬이 이 도구에 명시적으로 부여됨`
1. 둘 중 하나라도 실패: 호출 차단, 로깅, OreXis 센티넬에 보고

**구현:** `packages/shared/security_policy/src/` — 107개 테스트 어노테이션, 4개 tokio 테스트.

---

## 계층 3: 신뢰 수준 스킬 인가

**크레이트:** `domain_skills_permissions` (1,776 라인)

스킬은 기본 권한 범위를 결정하는 **신뢰 수준**으로 분류됩니다:

| 신뢰 수준 | 설명 | 기본 권한 |
| --- | --- | --- |
| `Builtin` | 플랫폼과 함께 제공됨 | 전체 도구 접근 |
| `Verified` | 관리자에 의해 검토 및 서명됨 | 읽기 + 쓰기 |
| `Community` | 사용자 제출 | 읽기 전용 |
| `Untrusted` | 동적으로 로드됨 | 도구 접근 없음 (exec 전용) |

각 스킬의 신뢰 수준은 로드 시점에 검증되고 캐시됩니다. 신뢰 수준을 상승시키려는 시도는 보안 이벤트로 로깅됩니다.

---

## 계층 4: 컨테이너 격리 (외부 링)

**크레이트:** `container` (5,742 라인)

모든 에이전트 실행은 다음과 같은 설정이 적용된 **Docker 또는 Podman 컨테이너** 내에서 발생합니다:

- 네트워크 네임스페이스 격리
- 읽기 전용 루트 파일시스템 (워크스페이스 마운트 제외)
- 시스템 호출을 제한하는 Seccomp 프로필
- 리소스 제한 (CPU, 메모리, PID 수)
- 호스트 Docker 소켓 접근 불가

**구현:** `packages/shared/container/src/` — 74개 테스트 어노테이션, 12개 tokio 테스트. Docker (Bollard API)와 Podman 모두 지원.

---

## 계층 5: OCI 샌드박스 (내부 링)

**크레이트:** `container_runtime` (3,645 라인)

Docker 컨테이너 내부에서, Entelecheia는 **두 번째 격리 계층**으로 Youki/libcontainer(데몬리스, 루트리스, OCI 준수 컨테이너 런타임)를 실행합니다. 이는 다음을 제공합니다:

- 루트리스 실행 (권한 상승 불가)
- Docker와 독립적인 네임스페이스 격리
- Cgroup v2 강제
- Seccomp 필터 (기본 거부)

**두 계층이 필요한 이유는?** Docker는 거친 격리(네트워크, 파일시스템)를 제공합니다. Youki는 세밀한 시스템 호출 필터링과 리소스 계정 관리를 제공합니다. Docker가 침해되더라도 Youki 샌드박스가 여전히 에이전트를 포함합니다.

---

## 계층 6: RBAC 접근 제어

**크레이트:** `domain_auth` (380 라인), shittim-chest `rbac` (1,736 라인)

모든 API 연산을 관장하는 역할 기반 접근 제어:

- **그룹:** 사용자는 그룹에 속하며, 그룹은 권한을 가집니다
- **권한:** 세분화된 권한 (리소스 유형별 읽기/쓰기/관리자)
- **워크스페이스 격리:** 사용자는 자신이 구성원인 워크스페이스만 접근 가능
- **크로스 워크스페이스 연산:** 명시적 관리자 권한 필요

---

## 계층 7: JWT 인증

**모듈:** shittim-chest `auth/jwt.rs` (264 라인)

- **알고리즘:** HS256 (HMAC-SHA256)
- **접근 토큰:** 단기 (설정 가능, 기본 15분)
- **갱신 토큰:** 사용 시 순환되는 장기 토큰
- **브라우저 클라이언트용 논스 기반 CSRF 보호**
- **인증 엔드포인트에 대한 속도 제한** (GCRA 알고리즘)

---

## 계층 8: API 키 암호화

**크레이트:** `aporia` (5,802 라인)

모든 LLM 프로바이더 API 키는 **AES-256-GCM**을 사용하여 저장 시 암호화됩니다:

- 암호화 연산당 고유 논스
- 마스터 시크릿에서 파생된 키 (환경 구성)
- 사용 후 메모리에서 평문 키 제로화
- 키 순환 지원

---

## 계층 9: 보안 센티넬 (OreXis)

**크레이트:** `orexis` (5,239 라인) — "면역 체계" 에이전트

OreXis는 다음과 같은 역할을 하는 레이어 1 에이전트입니다:

- 보안 취약점 및 라이선스 준수에 대한 **코드 감사**
- 등록된 보안 정책에 대한 **도구 호출 검사**
- 패턴별로 모든 에이전트의 도구 **차단/차단 해제**
- 비정상 패턴에 대한 에이전트 행동 **모니터링**

MCP 도구 (24개): `standard_check`, `compliance_report`, `audit_alignment`, `audit_legality`, `agent_integrity`, `security_audit`, `tool_block`, `tool_unblock`, `policy_register`, `policy_list` 등.

---

## 계층 10: IEPL 타입 안전 파이프라인

**크레이트:** `iepl` (2,670 라인), `iepl_engine` (1,228 라인), `skemma` (7,960 라인)

**Entelecheia 플러그인 언어** (IEPL) 파이프라인은 LLM 생성 코드와 네이티브 도구 디스패치 간의 타입 안전성을 보장합니다:

1. LLM이 ES 모듈 임포트를 사용하여 TypeScript 코드 생성
1. **SWC**가 TypeScript → JavaScript 트랜스파일 (구문 검증)
1. **Boa 엔진**이 샌드박스 컨텍스트에서 JavaScript 실행
1. ES 모듈 임포트가 `__native_dispatch` 호출로 해석됨
1. 각 디스패치는 완전한 타입 검사와 함께 `McpRouter`를 통해 라우팅됨

**완화되는 위협:** 비타입화 도구 호출을 통한 주입 공격 (도구 스키마가 런타임에만 검증되는 Python 기반 에이전트 프레임워크에서 흔함).

---

## 계층 11: 프로바이더 레지스트리 화이트리스트

**파일:** `configs/registries.toml` (337 라인)

Entelecheia는 15개 생태계에 걸쳐 신뢰할 수 있는 패키지 레지스트리의 **하드코딩된 화이트리스트**를 유지합니다:

crates.io, PyPI, npm, Go modules, Docker Hub, Maven Central, NuGet, RubyGems, Hackage, Alpine APK, Debian APT, GitHub, GitLab, `HuggingFace`, PyTorch.

화이트리스트에 없는 레지스트리로부터의 모든 패키지 임포트는 실행 전에 **컨테이너 수준에서 차단**됩니다.

---

## 계층 12: 프롬프트 주입 방어

**메커니즘:** IEPL 샌드박스 경계

LLM의 `exec` 출력은 다음에 접근할 수 없는 **격리된 Boa JS 컨텍스트**에서 실행됩니다:

- 호스트 파일시스템
- 네트워크 소켓
- 환경 변수
- 다른 에이전트의 상태

LLM에 반환되는 도구 출력은 **살균 처리**됩니다 — 바이너리 데이터는 base64 인코딩, 과도한 출력은 절단, 도구 결과의 잠재적 프롬프트 주입 패턴은 OreXis에 의해 플래그 지정됩니다.

---

## 계층 13: 속도 제한

**모듈:** shittim-chest `channel/rate_limit.rs` (118 라인)

**GCRA (Generic Cell Rate Algorithm)**를 사용한 사용자별, 채널별 속도 제한:

- 설정 가능한 버스트 크기와 지속 속도
- O(1) 조회를 위한 사용자별 DashMap
- 제한 초과 시 자동 백오프
- API 호출, 메시지 전송, 도구 호출에 대한 별도 제한

---

## 계층 14: 감사 추적

**크레이트:** `orexis`, `timeline` (3,096 라인)

모든 도구 호출, 에이전트 결정, 보안 이벤트는:

1. 전체 컨텍스트(에이전트 배지, 스킬명, 매개변수, 결과)와 함께 **타임라인**에 기록됨
1. 훼손 감지를 위해 이전 이벤트와 해시 연결됨
1. 설정 가능한 보존 기간으로 PostgreSQL에 영속화됨
1. CLI를 통해 조회 가능 (`entelecheia-cli trace-chain <badge>`)

---

## 다른 프레임워크와의 보안 비교

| 기능 | Entelecheia | OpenFANG | LangChain | Claude Code |
| --- |  ---  |  ---  |  ---  |  ---  |
| LLM 가시 도구 | **3 (exec-only)** | 53 (모두 가시) | 모두 가시 | 33 (모두 가시) |
| 컨테이너 격리 | **이중 계층** (Docker + Youki) | WASM 전용 | 없음 | OS 수준 (Seatbelt/Landlock) |
| 도구 권한 모델 | **이중 인가** | RBAC | 없음 | 없음 |
| 코드 감사 에이전트 | **OreXis (24개 도구)** | Loop guard | 없음 | 없음 |
| 타입 안전 디스패치 | **IEPL 파이프라인** | 직접 함수 호출 | 직접 함수 호출 | 직접 함수 호출 |
| 패키지 화이트리스트 | **15개 레지스트리** | 없음 | 없음 | 없음 |
| 감사 추적 | 해시 연결 타임라인 | Merkle 해시 체인 | 없음 | 없음 |

---

## 위협 모델

### 범위 외

- 호스트 머신에 대한 물리적 접근
- 침해된 Docker/Podman 데몬 (신뢰 가정)
- 커널 익스플로잇 (사용자 공간 격리에 의해 완화되나 방지되지는 않음)
- Rust 크레이트 의존성에 대한 공급망 공격 (`cargo-deny`에 의해 부분적으로 완화)

### 수용된 위험

- Boa JS 엔진 취약점 (컨테이너 내에서 샌드박스됨)
- LLM 프로바이더 중단 (폴백 실행 경로 없음)
- PostgreSQL 데이터 손상 (백업으로 완화, 방지되지 않음)

---

## 취약점 보고

취약점 보고 프로세스는 [SECURITY.md](../SECURITY.md)를 참조하십시오.

## 라이선스

이 보안 아키텍처는 Entelecheia의 일부이며, [BUSL-1.1](../LICENSE)에 따라 라이선스가 부여됩니다.
