+++
title = "모델 계층화 시스템 설계"
description = """모델 계층화 시스템은 태스크 복잡도에 따라 적절한 모델 계층을 할당하는 지능형 모델 선택 메커니즘으로, 품질을 보장하면서 자원 활용을 극대화합니다"""
lang = "ko"
category = "design"
subcategory = "core"
+++

# 모델 계층화 시스템 설계

## 개요

모델 계층화 시스템은 태스크 복잡도에 따라 적절한 모델 계층을 할당하는 지능형 모델 선택 메커니즘으로, 품질을 보장하면서 자원 활용을 극대화합니다.

> **관련 문서**: 본 문서에서 정의된 3계층 모델 시스템은 [자기 진화 루프 시스템](04-self-evolution-loop.md)의 기반입니다.

## 핵심 원칙

### 3계층 모델 시스템

```mermaid
graph TB
    subgraph 모델 계층
        T1[T1 심층 사고<br/>복잡한 추론]
        T2[T2 일반 사고<br/>표준 태스크]
        T3[T3 기본 사고<br/>원자적 연산]
    end

    T1 --> |하향| T2
    T2 --> |하향| T3
    T3 --> |파인튜닝 진화| T3_Fine[파인튜닝된 T3]
```

### 계층 비교

| 계층 | 포지셔닝 | 비용 | 대표적 시나리오 |
| --- | --- | --- | --- |
| T1 (심층) | 복잡한 추론, 의사 결정 | 최고 | 아키텍처 설계, 문제 분석 |
| T2 (일반) | 표준 태스크 | 중간 | 코드 작성, 문서 생성 |
| T3 (기본) | 원자적 연산 | 최저 | 파일 읽기, 형식 변환 |

## 모델 선택 메커니즘

### 선택 프로세스

```mermaid
flowchart TD
    Request[태스크 요청] --> Parse[level 필드 파싱]
    Parse --> Filter{계층 일치 모델 필터링}
    Filter --> |가용 모델 있음| Check{할당량 확인}
    Filter --> |가용 모델 없음| Downgrade[하향 시도]
    Check --> |할당량 충분| Select[최고 우선순위 선택]
    Check --> |할당량 소진| Next[다음 모델 시도]
    Select --> Execute[태스크 실행]
    Downgrade --> Filter
    Next --> Check
```

### 하향 전략

```mermaid
stateDiagram-v2
    [*] --> Deep: deep 태스크
    Deep --> Normal: deep 모델 불가용
    Normal --> Basic: normal 모델 불가용
    Basic --> [*]: 실행 또는 오류

    Deep --> [*]: 실행 성공
    Normal --> [*]: 실행 성공
```

## 설정 메커니즘

### 스킬/MCP 계층 어노테이션

각 스킬과 MCP 도구는 `level` 필드를 통해 필요한 모델 계층을 선언합니다:

```mermaid
graph LR
    subgraph 설정 계층
        S[스킬 설정]
        M[MCP 설정]
    end

    subgraph 레벨 필드
        L[level: deep/normal/basic]
    end

    S --> L
    M --> L
    L --> |런타임| Select[모델 선택기]
```

### 우선순위 제어

```mermaid
graph LR
    subgraph 우선순위 요소
        A[사용자 설정 우선순위]
        B[모델 계층 일치]
        C[할당량 상태]
    end

    A --> |최고 가중치| Sort[정렬]
    B --> |차순위 가중치| Sort
    C --> |필터 조건| Filter[필터]
    Filter --> Sort
    Sort --> Select[모델 선택]
```

## 다른 모듈과의 관계

```mermaid
graph TB
    A[모델 계층화 시스템] --> B[자기 진화 루프]
    A --> C[주기 사용량 통계]
    A --> D[비용 보고서]

    B --> |파인튜닝 대상| A
    C --> |선택 기준| A
    D --> |비용 데이터| A
```

## 설계 고려 사항

### 비용 최적화

- 하위 계층 모델 우선 사용
- 자동 하향으로 태스크 실패 방지
- 할당량 모니터링 알림

### 품질 보증

- 복잡한 태스크는 높은 계층 요구
- 하향 시 실행 가능성 검증 필요
- 실패 시 자동 재시도

### 확장성

- 사용자 정의 계층 지원
- 유연한 우선순위 구성
- 플러그인 가능한 선택 전략
