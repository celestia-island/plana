+++
title = "플랫폼 설계 문서"
description = """크로스 프로젝트(플랫폼 수준) 설계 문서. 프로젝트별 core/webui/router 하위 카테고리와 달리, 이곳의 문서들은 세 프로젝트(entelecheia, shittim-chest, evernight)에 걸친 관심사를 다룹니다 — 예를 들어 세 프로젝트가 공유하는 통합 감독, 롤링 업데이트 및 복제 아키텍처."""
lang = "ko"
category = "design"
subcategory = "platform"
+++

# 플랫폼 설계 문서

> **범위.** 이 문서들은 *플랫폼 수준*입니다. `core`(entelecheia),
> `webui`(shittim-chest), `router`(evernight)를 가로지릅니다. 프로젝트별
> 설계는 각자의 하위 카테고리에 있습니다.

## 색인

| 문서 | 요약 |
| --- | --- |
| [통합 감독, 롤링 업데이트 및 복제 아키텍처](supervision-and-rolling-update.md) | 세 프로젝트가 공유하는 단일 감독 트리(supervision tree) 골격: 통일된 시그널/드레인 시맨틱, 무정지 인계를 위한 systemd socket activation, 플러그 가능한 조정 잠금 trait, 그리고 동일한 Worker + Supervisor 원시 타입 위에 구축된 두 가지 내결함성 전략(Replica = 로드 밸런싱 ⊃ 롤링 업데이트; Leader/Follower = 엣지 HA). |

## 언어 디렉터리

| 코드 | 언어 |
| --- | --- |
| `en/` | 영어(정본) |
| `zhs/` | 중국어 간체 |
| `zht/` | 중국어 번체 |
| `ja/` | 일본어 |
| `ko/` | 한국어 |
| `fr/` | 프랑스어 |
| `es/` | 스페인어 |
| `ru/` | 러시아어 |
