+++
title = "프로바이더 TOML 설정 시스템 설계"
description = """프로바이더 TOML 설정 시스템은 모든 LLM 프로바이더 구성을 하드코딩된 값에서 TOML 설정 파일로 마이그레이션하여, 설정과 코드를 분리하고 유지보수성과 확장성을 향상시킵니다"""
lang = "ko"
category = "design"
subcategory = "core"
+++

# 프로바이더 TOML 설정 시스템 설계

## 개요

프로바이더 TOML 설정 시스템은 모든 LLM 프로바이더 구성을 하드코딩된 값에서 TOML 설정 파일로 마이그레이션하여, 설정과 코드를 분리하고 유지보수성과 확장성을 향상시킵니다.

## 핵심 목표

| 목표 | 설명 |
| --- | --- |
| 유지보수성 | 설정이 코드와 분리되어 변경 시 재컴파일 불필요 |
| 확장성 | 새 프로바이더 추가 시 TOML 파일만 추가하면 됨 |
| 가독성 | 설정 파일이 명확하고 이해하기 쉬움 |
| 재사용성 | 설정을 다양한 환경에서 공유 가능 |

## 아키텍처 설계

### 설정 로딩 프로세스

```mermaid
flowchart TB
    subgraph 초기화 단계
        A[애플리케이션 시작] --> B[res/ 디렉터리 스캔]
        B --> C[모든 .toml 파일 로드]
        C --> D[TOML 구조 파싱]
    end

    subgraph 검증 단계
        D --> E{설정 완전성 검증}
        E -->|통과| F[설정 캐시에 저장]
        E -->|실패| G[오류 로그]
        G --> H[기본 설정 사용]
    end

    subgraph 런타임
        F --> I[프로바이더 요청]
        I --> J[캐시에서 설정 조회]
        J --> K[ProviderConfig 반환]
    end
```

### 설정 계층 구조

```mermaid
graph TB
    subgraph ProviderConfig
        A[프로바이더 정보]
        B[API 설정]
        C[제한 설정]
        D[가격 설정]
        E[기능 설정]
        F[모델 목록]
    end

    A --> A1[id, name, type, protocol]
    B --> B1[base_url, endpoints, auth]
    C --> C1[동시성 제한, 속도 제한, 타임아웃]
    D --> D1[과금 모드, 할당량 정보]
    E --> E1[스트리밍, 비전, function_calling]
    F --> F1[ModelConfig 목록]

    subgraph ModelConfig
        F1 --> M1[id, name, context_window]
        F1 --> M2[기능 지원 플래그]
        F1 --> M3[가격 정보]
        F1 --> M4[벤치마크 데이터]
    end
```

## 설정 우선순위

```mermaid
graph LR
    A[사용자 설정] -->|최고 우선순위| D[유효 설정]
    B[커뮤니티 설정] -->|중간 우선순위| D
    C[공식 설정] -->|기본 우선순위| D

    style A fill:#90EE90
    style B fill:#FFD700
    style C fill:#87CEEB
```

### 우선순위 병합 규칙

| 계층 | 출처 | 설명 |
| --- | --- | --- |
| 1 | 공식 설정 | 프로바이더 공식 문서 데이터, 기본값으로 사용 |
| 2 | 커뮤니티 설정 | 커뮤니티 기여 최적화 설정, 공식 데이터 덮어씀 |
| 3 | 사용자 설정 | 사용자 정의 설정, 최고 우선순위 |

## 가격 모델

```mermaid
stateDiagram-v2
    [*] --> PayAsYouGo: 사용량 기반 과금
    [*] --> OneTime: 일회성 구매
    [*] --> Periodic: 정기 할당량
    [*] --> Free: 무료

    PayAsYouGo --> 사용량 측정
    OneTime --> 잔액 확인
    Periodic --> 기간 할당량 확인
    Free --> 무제한
```

### 가격 모델 비교

| 모델 | 적용 시나리오 | 특성 |
| --- | --- | --- |
| PayAsYouGo | OpenAI, Anthropic | 토큰당 과금, 실시간 차감 |
| OneTime | 선불 패키지 | 할당량 사전 구매, 소진 시까지 사용 |
| Periodic | GLM 중국 등 | 주기적 할당량 초기화 |
| Free | Ollama 로컬 모델 | 비용 제한 없음 |

## 프로바이더 유형 분류

```mermaid
graph TB
    subgraph 클라우드 프로바이더
        A[OpenAI 호환 프로토콜]
        B[Anthropic 프로토콜]
        C[Google Gemini 프로토콜]
    end

    subgraph 로컬 프로바이더
        D[Ollama]
        E[LocalAI]
    end

    subgraph 사용자 정의 프로바이더
        F[사용자 정의 엔드포인트]
    end

    A --> A1[OpenAI, DeepSeek, Qwen]
    B --> B1[Claude 시리즈]
    C --> C1[Gemini 시리즈]
```

## 핫 리로드 메커니즘

```mermaid
sequenceDiagram
    participant FS as 파일 시스템
    participant Watcher as 설정 감시자
    participant Cache as 설정 캐시
    participant App as 애플리케이션

    FS->>Watcher: 파일 변경 이벤트
    Watcher->>Watcher: 변경 내용 파싱
    Watcher->>Cache: 캐시 갱신
    Cache->>App: 설정 갱신 알림 전송
    App->>App: 새 설정 적용
```

## 오류 처리 전략

```mermaid
flowchart TB
    A[설정 로딩] --> B{파싱 성공?}
    B -->|예| C[설정 검증]
    B -->|아니오| D[파싱 오류 로그]

    C --> E{검증 통과?}
    E -->|예| F[캐시에 저장]
    E -->|아니오| G[검증 오류 로그]

    D --> H[기본 설정 사용]
    G --> H

    F --> I[정상 사용]
    H --> I
```

## 확장성 설계

### 새 프로바이더 추가

```mermaid
flowchart LR
    A[TOML 파일 생성] --> B[프로바이더 정보 정의]
    B --> C[API 엔드포인트 구성]
    C --> D[모델 목록 추가]
    D --> E[가격 정보 설정]
    E --> F[애플리케이션 재시작]
    F --> G[설정 자동 로드]
```

### 설정 검증 규칙

| 필드 | 검증 규칙 | 오류 처리 |
| --- | --- | --- |
| provider.id | 비어 있지 않음, 고유함 | 로딩 거부, 오류 로그 |
| api.base_url | 유효한 URL 형식 | 기본값 사용 |
| models[].id | 비어 있지 않음 | 해당 모델 건너뜀 |
| pricing.model | Enum 값 확인 | 기본값 PayAsYouGo |

## 보안 고려 사항

```mermaid
flowchart TB
    subgraph 민감 정보 처리
        A[API 키] --> B[암호화 저장]
        B --> C[메모리 내 사용]
        C --> D[로그 마스킹]
    end

    subgraph 접근 제어
        E[설정 읽기] --> F{권한 확인}
        F -->|권한 있음| G[설정 반환]
        F -->|권한 없음| H[접근 거부]
    end
```

## 향후 확장

| 기능 | 설명 | 우선순위 |
| --- | --- | --- |
| 설정 핫 리로드 | 런타임에 외부 설정 파일 로드 | 높음 |
| 설정 검증 | 시작 시 설정 완전성 검증 | 높음 |
| 설정 병합 | 사용자 설정이 기본 설정 덮어씀 | 중간 |
| 설정 임포트/익스포트 | 설정 파일 임포트/익스포트 지원 | 중간 |
| 에이전트 갱신 | 공식 문서에서 설정 자동 갱신 | 낮음 |

# 프로바이더 메타데이터 관리 설계

## 개요

프로바이더 메타데이터 관리 시스템은 공식 LLM 프로바이더 문서에서 설정 정보를 동적으로 가져와, 설정 데이터의 자동화된 갱신 및 검증을 가능하게 합니다.

## 핵심 문제

현재 구현은 하드코딩된 사용량 통계를 포함하고 있으며 동적 프로바이더 데이터 지원이 부족합니다. 자동화된 메타데이터 획득 및 관리 메커니즘을 구축해야 합니다.

## 아키텍처 설계

### 데이터 흐름 아키텍처

```mermaid
flowchart TB
    subgraph 데이터 소스
        A[공식 문서]
        B[API 엔드포인트]
        C[커뮤니티 기여]
    end

    subgraph 수집 계층
        D[설정 에이전트]
        E[웹 스크래퍼]
        F[API 클라이언트]
    end

    subgraph 처리 계층
        G[데이터 파서]
        H[검증 엔진]
        I[병합 전략]
    end

    subgraph 저장 계층
        J[설정 데이터베이스]
        K[캐시 계층]
    end

    A --> D
    B --> F
    C --> D
    D --> G
    E --> G
    F --> G
    G --> H
    H --> I
    I --> J
    J --> K
```

### 설정 우선순위 모델

```mermaid
graph TB
    subgraph 우선순위 계층
        A[사용자 설정] -->|최고| D[유효 설정]
        B[커뮤니티 설정] -->|중간| D
        C[공식 설정] -->|기본| D
    end

    subgraph 병합 규칙
        D --> E[필드 수준 덮어쓰기]
        E --> F[높은 우선순위 값 유지]
    end
```

## 메타데이터 구조

### 프로바이더 설정 계층 구조

```mermaid
classDiagram
    class ProviderConfig {
        +provider_id: String
        +display_name: String
        +available_models: List~ModelConfig~
        +default_model: String
        +pricing_model: PricingModel
        +usage_type: UsageType
        +api_endpoint: String
    }

    class ModelConfig {
        +model_id: String
        +model_name: String
        +context_window: u64
        +max_output_tokens: u64
        +supports_vision: bool
        +supports_function_calling: bool
    }

    class PricingModel {
        <<enumeration>>
        OneTime
        Periodic
        PayAsYouGo
    }

    class UsageType {
        <<enumeration>>
        Metered
        Quota
        Unlimited
    }

    ProviderConfig --> ModelConfig
    ProviderConfig --> PricingModel
    ProviderConfig --> UsageType
```

### 설정 출처 분류

| 출처 유형 | 설명 | 신뢰도 | 갱신 빈도 |
| --- | --- | --- | --- |
| 공식 | 프로바이더 공식 문서 | 높음 | 자동 주기적 |
| 커뮤니티 | 커뮤니티 기여 데이터 | 중간 | 수동 갱신 |
| 사용자덮어쓰기 | 사용자 사용자 정의 | 최고 | 실시간 |

## 에이전트 수집 시스템

### 수집 프로세스

```mermaid
sequenceDiagram
    participant Scheduler as 스케줄러
    participant Agent as 설정 에이전트
    participant Source as 데이터 소스
    participant Parser as 파서
    participant Validator as 검증기
    participant DB as 데이터베이스

    Scheduler->>Agent: 수집 태스크 트리거
    Agent->>Source: 공식 문서 요청
    Source-->>Agent: HTML/JSON 반환
    Agent->>Parser: 콘텐츠 파싱
    Parser-->>Agent: 구조화된 데이터
    Agent->>Validator: 데이터 검증
    Validator-->>Agent: 검증 결과
    Agent->>DB: 설정 저장
    DB-->>Agent: 저장 성공
    Agent-->>Scheduler: 태스크 완료
```

### 프로바이더 에이전트 책임

```mermaid
flowchart LR
    subgraph OpenAI 에이전트
        A1[모델 목록 조회]
        A2[가격 정보 파싱]
        A3[속도 제한 추출]
    end

    subgraph Anthropic 에이전트
        B1[Claude 모델 조회]
        B2[컨텍스트 윈도우 파싱]
        B3[기능 정보 추출]
    end

    subgraph GLM 에이전트
        C1[GLM 모델 조회]
        C2[할당량 정보 파싱]
        C3[초기화 주기 추출]
    end
```

## 데이터 검증 메커니즘

### 검증 프로세스

```mermaid
flowchart TB
    A[설정 데이터 수신] --> B{형식 검증}
    B -->|통과| C{논리 검증}
    B -->|실패| D[오류 로그]

    C -->|통과| E{완전성 검증}
    C -->|실패| D

    E -->|통과| F{일관성 검증}
    E -->|실패| G[기본값 채움]

    F -->|통과| H[설정 수락]
    F -->|실패| I[검토 대상 표시]

    G --> F
    D --> J[설정 거부]
```

### 검증 규칙

| 검증 유형 | 확인 내용 | 실패 처리 |
| --- | --- | --- |
| 형식 검증 | 데이터 타입, 필드 형식 | 거부 및 로그 |
| 논리 검증 | 값 범위, Enum 값 | 기본값 사용 |
| 완전성 검증 | 필수 필드 존재 | 기본값 채움 |
| 일관성 검증 | 필드 간 관계 정확성 | 검토 대상 표시 |

## 설정 병합 전략

### 필드 수준 병합

```mermaid
flowchart TB
    subgraph 입력
        A[공식 설정]
        B[커뮤니티 설정]
        C[사용자 설정]
    end

    subgraph 병합 프로세스
        D[필드별 우선순위]
        E[non-null 값 유지]
        F[결과 검증]
    end

    A --> D
    B --> D
    C --> D
    D --> E
    E --> F
    F --> G[유효 설정]
```

### 병합 예시

| 필드 | 공식 값 | 커뮤니티 값 | 사용자 값 | 최종 값 |
| --- | --- | --- | --- | --- |
| context_window | 128000 | - | 64000 | 64000 |
| max_concurrent | 100 | 50 | - | 50 |
| pricing_model | PayAsYouGo | - | - | PayAsYouGo |

## 사용자 설정 인터페이스

### 설정 파일 구조

```mermaid
graph TB
    subgraph 사용자 설정 파일
        A[프로바이더 표시 이름]
        B[사용 유형 설정]
        C[할당량 제한]
        D[동시성 제어]
        E[컨텍스트 관리]
        F[모델 덮어쓰기]
    end

    A --> A1[사용자 정의 표시 이름]
    B --> B1[metered/quota/unlimited]
    C --> C1[데이터 제한/복구 주기]
    D --> D1[최대 동시성]
    E --> E1[이론적 제한/실용적 제한]
    F --> F1[사용자 정의 모델 목록]
```

## 스케줄 갱신 메커니즘

```mermaid
sequenceDiagram
    participant Timer as 타이머
    participant Queue as 태스크 큐
    participant Agent as 에이전트 풀
    participant DB as 데이터베이스

    Timer->>Queue: 갱신 태스크 추가
    Queue->>Agent: 태스크 할당

    loop 각 프로바이더
        Agent->>Agent: 최신 설정 조회
        Agent->>DB: 변경 사항 비교
        alt 변경 있음
            DB->>DB: 설정 갱신
            DB->>DB: 변경 사항 로그
        else 변경 없음
            DB->>DB: 확인 시간 갱신
        end
    end

    Agent-->>Queue: 태스크 완료
```

## 오류 처리

### 수집 실패 처리

```mermaid
flowchart TB
    A[수집 실패] --> B{실패 유형}
    B -->|네트워크 오류| C[재시도 메커니즘]
    B -->|파싱 오류| D[로그 및 건너뜀]
    B -->|검증 오류| E[검토 대상 표시]

    C --> F{재시도 횟수}
    F -->|초과 안 됨| G[지연 재시도]
    F -->|초과됨| H[캐시 데이터 사용]

    G --> A
    D --> I[다음 항목 계속]
    E --> J[수동 검토 큐]
```

## 확장성 설계

### 새 프로바이더 추가

```mermaid
flowchart LR
    A[에이전트 정의] --> B[수집 인터페이스 구현]
    B --> C[파싱 규칙 구성]
    C --> D[스케줄러에 등록]
    D --> E[수집 시작]
```

### 확장 포인트

| 확장 유형 | 설명 | 구현 방식 |
| --- | --- | --- |
| 새 프로바이더 | 새 설정 소스 추가 | 프로바이더 에이전트 인터페이스 구현 |
| 새 필드 | 설정 구조 확장 | 데이터 모델 및 검증 규칙 갱신 |
| 새 검증 규칙 | 검증 로직 추가 | 검증기 구현 추가 |

## 레이어3 에이전트 구현

### ProviderScratch 에이전트

`ProviderScratch`는 최초의 레이어3 공식 에이전트로, 스크래핑 기능의 예시 구현 역할을 합니다.

```mermaid
flowchart TB
    subgraph ProviderScratch 에이전트
        A[에이전트 진입] --> B{실행 모드}
        B -->|TUI 모드| C[대화형 인터페이스]
        B -->|CI 모드| D[자동 실행]

        C --> E[프로바이더 선택]
        D --> F[환경 변수 읽기]

        E --> G[스킬 호출]
        F --> G

        G --> H[문서 스크래핑]
        H --> I[데이터 파싱]
        I --> J[TOML 생성]

        J --> K{커밋 확인?}
        K -->|예| L[워크스페이스에 쓰기]
        K -->|아니오| M[변경 사항 폐기]

        L --> N[사용자 커밋 요청]
    end
```

### 스킬 아키텍처

각 프로바이더는 독립적인 스킬에 대응합니다:

```mermaid
graph LR
    subgraph 스킬
        A[openai]
        B[anthropic]
        C[glm]
        D[deepseek]
        E[qwen]
        F[gemini]
    end

    subgraph 공유 구성 요소
        G[문서 스크래퍼]
        H[데이터 파서]
        I[TOML 생성기]
    end

    A --> G
    B --> G
    C --> G
    D --> G
    E --> G
    F --> G

    G --> H
    H --> I
```

### 디렉터리 구조

```mermaid
flowchart LR
    Root[".amphoreus/provider_scratch/"]
    AT["agent.toml"]
    OV["overview/"]
    SK["skills/"]
    Root --> AT
    Root --> OV
    Root --> SK
    OV --> ZH["zhs.md"]
    SK --> OA["openai/"]
    SK --> AN["anthropic/"]
    SK --> GL["glm/"]
    SK --> DS["deepseek/"]
    SK --> QW["qwen/"]
    SK --> GE["gemini/"]
    OA --> OAP["prompt.md"]
    AN --> ANP["prompt.md"]
    GL --> GLP["prompt.md"]
    DS --> DSP["prompt.md"]
    QW --> QWP["prompt.md"]
    GE --> GEP["prompt.md"]
```

### CI 자동화

```mermaid
flowchart LR
    A[스케줄 트리거] --> B[코드 체크아웃]
    B --> C[ProviderScratch 실행]
    C --> D{변경 감지}
    D -->|변경 있음| E[브랜치 생성]
    E --> F[변경 사항 커밋]
    F --> G[PR 생성]
    G --> H[리뷰 대기]
    D -->|변경 없음| I[완료]
```

### 환경 변수

| 변수명 | 설명 |
| --- | --- |
| `AMPHOREUS_PROVIDER_SCRATCH_PROVIDERS` | 스크래핑할 프로바이더 목록 |
| `AMPHOREUS_PROVIDER_SCRATCH_OUTPUT_DIR` | 출력 디렉터리 경로 |
| `AMPHOREUS_PROVIDER_SCRATCH_GIT_BRANCH` | 대상 Git 브랜치 |
| `AMPHOREUS_PROVIDER_SCRATCH_DRY_RUN` | 드라이 런 전용 |

## 향후 계획

| 기능 | 설명 | 우선순위 |
| --- | --- | --- |
| 설정 버전 관리 | 설정 변경 이력 추적 | 높음 |
| 변경 알림 | 설정 갱신 시 사용자에게 알림 | 중간 |
| 설정 롤백 | 이전 버전으로 롤백 지원 | 중간 |
| 스마트 추천 | 사용 패턴 기반 설정 추천 | 낮음 |
| GitHub 순회 에이전트 | 설정 갱신을 위한 PR 자동 생성 | 높음 |
