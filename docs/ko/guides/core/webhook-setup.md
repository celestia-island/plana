+++
title = "Webhook 플랫폼 설정"
description = """> 현재 webhook 레이아웃 및 통합 범위 설명"""
lang = "ko"
category = "guides"
subcategory = "core"
+++

# Webhook 플랫폼 설정

> 현재 webhook 레이아웃 및 통합 범위 설명

## 개요

저장소에는 코드 호스팅 플랫폼 및 채팅 플랫폼용 webhook 통합이 이미 포함되어 있지만, 전체적으로는 전환 단계에 있으며 완전히 통합되고 성숙하게 정형화된 방안은 아닙니다.

현재 디렉터리 구조에는 다음이 동시에 존재합니다:

- 이전 플랫폼별 디렉터리: `plugins/github-webhook/github/`, `gitee/`, `gitlab/`, `telegram/`, `qq/`, `lark/`
- 새로운 TypeScript 구현: `plugins/github-webhook/ts/`

TypeScript 패키지는 현재 다음을 지원합니다:

- GitHub
- Gitee
- GitLab
- Feishu / Lark
- QQ
- Discord
- Telegram

## 현재 할 수 있는 일

- webhook 또는 bot 이벤트 수신
- WebSocket 또는 HTTP 보조 호출을 통해 Scepter에 이벤트 전달
- TypeScript 서비스에서 `/health` 헬스 체크 엔드포인트 제공

## 현재 기본적으로 보장할 수 없는 것

- 모든 플랫폼에 대한 통일되고 안정적인 배포 방안
- 각 플랫폼이 완전한 issue 기반 skill chain을 형성했는지 여부
- 모든 플랫폼 통합이 동일한 성숙도에 도달했는지 여부

## TypeScript 패키지

위치: `plugins/github-webhook/ts/`

개발 실행 방법:

```bash
cd plugins/github-webhook/ts
npm install
npm run dev
```

프로덕션 빌드 방법:

```bash
cd plugins/github-webhook/ts
npm run build
npm start
```

## 주요 환경 변수

- `PORT`: webhook 서비스 포트, 기본값 `8000`
- `SCEPTER_URL`: HTTP 전달 주소, 기본값 `http://localhost:8424`
- `SCEPTER_WS_URL`: WebSocket 전달 주소, 기본값 `ws://localhost:8424/ws`

## 사용 권장 사항

webhook 기능은 "이미 존재하지만 성숙도가 균일하지 않다"고 볼 수 있습니다. 특정 플랫폼에 의존하는 경우, 먼저 `plugins/github-webhook/` 아래 해당 라우터 또는 bot의 실제 구현을 확인한 후, 안정적인 프로덕션 사용 가능으로 설명할지 결정하십시오.
