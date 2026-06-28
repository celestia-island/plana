+++
title = "아키텍처 결정 기록 (ADR)"
description = """이 디렉터리는 Entelecheia 개발 과정에서 이루어진 주요 아키텍처 결정을 기록합니다. 각 ADR은 무엇이 결정되었는지, 왜 그렇게 결정되었는지, 그리고 어떤 절충이 고려되었는지�"""
lang = "ko"
category = "design"
subcategory = "core"
+++

# 아키텍처 결정 기록 (ADR)

이 디렉터리는 Entelecheia 개발 과정에서 이루어진 주요 아키텍처 결정을 기록합니다. 각 ADR은 **무엇이** 결정되었는지, **왜** 그렇게 결정되었는지, 그리고 어떤 **절충**이 고려되었는지를 설명합니다.

ADR은 [Michael Nygard ADR 템플릿](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)을 따릅니다. 게시된 이후에는 불변입니다 — 대체된 결정은 그에 따라 표시됩니다.

## 색인

| ADR | 제목 | 상태 |
| --- | --- | --- |
| [ADR-001](en/exec-only-microkernel-tool-surface.md) | Exec-Only 마이크로커널 도구 표면 | 승인됨 |
| [ADR-002](en/boa-javascript-engine.md) | 임베디드 JavaScript 엔진으로서의 Boa | 승인됨 |
| [ADR-003](en/postgresql-pgvector-storage.md) | 통합 데이터 저장소를 위한 PostgreSQL + PgVector | 승인됨 |
| [ADR-004](en/layered-crate-workspace.md) | 60+ 크레이트 계층형 워크스페이스 아키텍처 | 승인됨 |
| [ADR-005](en/container-sandboxed-agent-execution.md) | COSMOS를 통한 컨테이너 샌드박스 에이전트 실행 | 승인됨 |

## 언어 디렉터리

| 코드 | 언어 |
| --- | --- |
| `en/` | 영어 (권위) |
| `zhs/` | 중국어 간체 |
| `zht/` | 중국어 번체 |
| `ja/` | 일본어 |
| `ko/` | 한국어 |
| `fr/` | 프랑스어 |
| `es/` | 스페인어 |
| `ru/` | 러시아어 |
