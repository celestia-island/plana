+++
title = "Shittim Chest (什亭之匣) 소개"
description = """버전 0.1.0"""
lang = "ko"
category = "architecture"
subcategory = "webui"
+++

# Shittim Chest (什亭之匣)

**버전 0.1.0**

Shittim Chest는 [entelecheia](https://github.com/celestia-island/entelecheia) 멀티 에이전트 협업 플랫폼을 위한 사용자 대면 셸로, Rust와 Vue 3로 구축되었습니다.

## 아키텍처

Shittim Chest는 완전한 사용자 경험을 제공하기 위해 함께 작동하는 여러 구성 요소로 이루어져 있습니다:

- **arona** — 현재 사용 중인 채팅 UI로, 스트리밍 응답, 이미지 생성, 에이전트 상태 모니터링, 사고 창, 원격 장치 뷰어, 다국어 지원을 제공합니다.
- **`shittim_chest`** — 인증(JWT + OAuth), 독립적 LLM 라우팅, 채팅 API, 이미지 생성, 웹훅 수신, scepter 프록시, 원격 장치 시그널링을 처리하는 통합 Rust + Axum 백엔드입니다.

## Entelecheia와의 관계

[entelecheia](https://github.com/celestia-island/entelecheia)는 핵심 멀티 에이전트 오케스트레이션 엔진입니다. 에이전트 런타임(scepter, 13개 특화 에이전트, Cosmos/IEPL 런타임)을 제공합니다. Shittim Chest는 사용자가 직접 상호작용하는 모든 것 — 신원, 표현, 통신을 처리합니다.

두 프로젝트는 설계상 분리되어 있습니다: entelecheia는 에이전트 오케스트레이션을 관리하고, shittim-chest는 사용자 신원과 표현을 관리합니다. 이들은 JWT 인증 HTTP/WebSocket을 통해 통신합니다. 로그인 자격 증명은 shittim_chest_db에 저장되고, 권한 및 신원 데이터는 entelecheia_db에 저장됩니다. 이 분리를 통해 프론트엔드 셸이 에이전트 코어와 독립적으로 발전할 수 있습니다.

## Hikari와의 관계

[hikari](https://github.com/celestia-island/hikari)는 Celestia Island 생태계의 게이트웨이 및 라우팅 계층입니다. 모든 외부 트래픽의 진입점으로, shittim-chest, entelecheia 및 기타 서비스 간의 요청 라우팅, 로드 밸런싱, API 게이트웨이 기능을 처리합니다.

## Tairitsu와의 관계

[tairitsu](https://github.com/celestia-island/tairitsu)는 Celestia Island 생태계의 크로스 플랫폼 네이티브 애플리케이션 프레임워크입니다. arona를 네이티브 애플리케이션으로 감싸는 Tauri 기반 데스크톱 및 모바일 클라이언트와 개발 워크플로우를 지원하는 브라우저 자동화 및 테스팅 인프라를 제공합니다.

## 라이선스

Shittim Chest는 **Business Source License 1.1 (BSL-1.1)**에 따라 라이선스가 부여됩니다.

**비상업적 사용** — 내부 운영, 학술 연구, 교육, 개인 학습, 평가, 정부 및 공공 서비스, 교육적 사용을 포함하여 — 부여된 권리는 **Synthetic Source License 1.0 (SySL-1.0)**("무료 사용 라이선스")과 동등합니다. 이러한 목적으로 소프트웨어를 자유롭게 사용, 연구, 수정, 실행할 수 있습니다.

**상업적 사용** — 소프트웨어를 제3자에게 호스팅 서비스로 제공하거나, 독립형 제품으로 재배포하거나, 상업적 제공의 핵심 구성 요소로 사용하는 경우 — Licensor로부터 별도의 상업 라이선스가 필요합니다.

자세한 내용은 [전체 라이선스 텍스트](https://github.com/celestia-island/shittim-chest/blob/main/LICENSE)를 참조하십시오.

---

[Celestia Island](https://github.com/celestia-island)가 ❤로 제작
