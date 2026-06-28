+++
title = "웹훅 설정 가이드"
description = """> 대상: 외부 서비스를 shittim-chest와 통합하는 관리자."""
lang = "ko"
category = "guides"
subcategory = "webui"
+++

# 웹훅 설정 가이드

> **대상**: 외부 서비스를 shittim-chest와 통합하는 관리자.
> **최종 업데이트**: 2026-05-25

## 개요

웹훅을 통해 외부 서비스(GitHub, GitLab, Gitee)가 shittim-chest에 실시간 이벤트를 보낼 수 있습니다. 이벤트는 검증, 파싱되어 적절한 에이전트로 디스패치하는 scepter로 전달됩니다.

```text
외부 서비스 → shittim_chest → scepter → 에이전트
```

shittim_chest는 기본적으로 지원되지 않는 서비스를 위한 사용자 정의 웹훅 엔드포인트도 지원합니다.

## GitHub 웹훅 설정

### 1단계: 환경 설정

`.env`에 웹훅 비밀키 설정:

```bash
WEBHOOK_GITHUB_SECRET=your-hmac-secret-here
WEBHOOK_PUBLIC_URL=https://your-domain.com
```

강력한 비밀키 생성:

```bash
openssl rand -hex 32
```

### 2단계: GitHub에서 웹훅 생성

1. 저장소 → **Settings** → **Webhooks** → **Add webhook**로 이동
1. **Payload URL**을 `https://your-domain.com/api/webhook/github`로 설정
1. **Content type**을 `application/json`으로 설정
1. **Secret**을 `WEBHOOK_GITHUB_SECRET`과 동일한 값으로 설정
1. 이벤트 선택: `push`, `pull_request`, `issues`, `issue_comment`
1. **Active**가 체크되었는지 확인
1. **Add webhook** 클릭

### 3단계: 확인

GitHub이 즉시 `ping` 이벤트를 보냅니다. **Recent Deliveries** 탭에서 `200` 응답을 확인하십시오.

## GitLab 웹훅 설정

### 1단계: 환경 설정

```bash
WEBHOOK_GITLAB_SECRET=your-gitlab-secret-token
```

### 2단계: GitLab에서 웹훅 생성

1. 프로젝트 → **Settings** → **Webhooks**로 이동
1. **URL**을 `https://your-domain.com/api/webhook/gitlab`로 설정
1. **Secret token**을 `WEBHOOK_GITLAB_SECRET`과 동일한 값으로 설정
1. 트리거 선택: `Push events`, `Merge request events`, `Issue events`
1. **Enable SSL verification**이 체크되었는지 확인 (HTTPS의 경우)
1. **Add webhook** 클릭

### 3단계: 확인

GitLab의 **Test** 버튼을 사용하여 테스트 이벤트 전송. 배달 성공을 확인하십시오.

## Gitee 웹훅 설정

Gitee (码云) 웹훅도 지원됩니다.

### 1단계: 환경 설정

Gitee는 HMAC 검증에 동일한 `WEBHOOK_GITLAB_SECRET`을 사용합니다(토큰을 폴백으로). 또는 비밀번호 기반 인증 사용 시 `WEBHOOK_GITEE_PASSWORD` 설정.

### 2단계: Gitee에서 웹훅 생성

1. 저장소 → **Management** → **Webhooks**로 이동
1. **URL**을 `https://your-domain.com/api/webhook/gitee`로 설정
1. **Password/Signing Key**를 동일한 비밀키로 설정
1. 이벤트 선택: `Push`, `Pull Request`, `Issues`
1. **Add** 클릭

## 사용자 정의 웹훅

shittim_chest는 `/api/webhook/custom/{name}`에서 일반 사용자 정의 웹훅 엔드포인트를 지원합니다. 사용자 정의 웹훅 소스를 추가하려면:

1. `.env`에 `WEBHOOK_PUBLIC_URL` 설정
1. 외부 서비스가 `https://your-domain.com/api/webhook/custom/{name}`으로 POST하도록 구성
1. 이벤트가 웹훅 이름을 이벤트 소스로 하여 scepter로 전달됨

코드 수준에서 새로운 웹훅 제공자를 통합하려면:

1. `packages/core/src/webhook.rs`에 핸들러 추가
1. 새 제공자에 대한 HMAC 또는 토큰 검증 구현
1. 사용자 정의 이벤트 형식 파싱 및 Unix 소켓을 통해 scepter로 전달

## IP 화이트리스트

shittim_chest는 알 수 없는 출처의 요청을 거부하기 위해 웹훅 소스에 대한 IP 화이트리스트를 지원합니다:

```bash
# .env
WEBHOOK_IP_WHITELIST=140.82.112.0/20,192.30.252.0/22  # GitHub IP
```

각 웹훅 제공자에 대한 CIDR 범위 구성. 화이트리스트 외 IP의 요청은 거부됩니다.

## 이벤트 유형

지원되는 이벤트 및 scepter 트리거로의 매핑:

| 소스 | 이벤트 | scepter `event_type` |
| --- | --- | --- |
| GitHub | `push` | `github.push` |
| GitHub | `pull_request` | `github.pull_request` |
| GitHub | `issues` | `github.issues` |
| GitHub | `issue_comment` | `github.issue_comment` |
| GitLab | `push` | `gitlab.push` |
| GitLab | `merge_request` | `gitlab.merge_request` |
| GitLab | `issues` | `gitlab.issues` |
| Gitee | `push` | `gitee.push` |
| Gitee | `pull_request` | `gitee.pull_request` |
| Gitee | `issues` | `gitee.issues` |

## 배달 로그

shittim_chest는 웹훅 이벤트의 배달 로그를 유지합니다. 중복 배달은 LRU 캐시(최대 10,000 배달 ID)를 사용하여 감지됩니다. 배달 로그 접근:

- **REST API**: `GET /api/webhook/deliveries`
- 관리자 패널: **Webhooks** → **Delivery Log**

## 보안

모든 웹훅은 서명 검증을 통과해야 합니다:

- **GitHub**: `X-Hub-Signature-256` 헤더 사용. `WEBHOOK_GITHUB_SECRET`으로 검증.
- **GitLab**: `X-Gitlab-Token` 헤더 사용. `WEBHOOK_GITLAB_SECRET`으로 검증.
- **Gitee**: 토큰 폴백이 있는 HMAC-SHA256 서명 사용.

유효한 서명이 없는 요청은 `401 Unauthorized`로 거부됩니다. 웹훅 비밀키를 클라이언트 측 코드나 로그에 절대 노출하지 마십시오.

## 테스트

관리자 패널을 사용하여 웹훅 통합 테스트:

1. 관리자 패널 로그인 (기본 `:3000`)
1. 사이드바에서 **Webhooks**로 이동
1. 배달 로그 및 구성 확인
1. 외부 서비스의 테스트 기능을 통해 엔드포인트 테스트

curl로 수동 테스트도 가능:

```bash
curl -X POST https://your-domain.com/api/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=<computed-hmac>" \
  -d '{"action":"push","ref":"refs/heads/main"}'
```

## 문제 해결

### 401 Unauthorized

**원인**: HMAC 서명 불일치 또는 IP가 화이트리스트에 없음.
**해결**: `.env`의 비밀키가 소스 플랫폼에 구성된 비밀키와 일치하는지 확인. 후행 공백이나 인코딩 문제 확인. IP 화이트리스트 구성 확인.

### 502 Bad Gateway

**원인**: scepter에 연결할 수 없음.
**해결**: `.env`의 `ENTELECHEIA_SCEPTER_URL` 및 `ENTELECHEIA_TUI_SOCK` 확인. scepter 인스턴스가 실행 중이고 Unix 소켓 경로에 접근 가능한지 확인.

### 이벤트가 에이전트에 도달하지 않음

**원인**: 이벤트 유형이 매핑되지 않았거나 에이전트가 처리하도록 구성되지 않음.
**해결**: 파싱된 `event_type`에 대한 백엔드 로그 확인. 대상 에이전트에 해당 이벤트에 대한 핸들러가 등록되었는지 확인. API 또는 관리자 패널을 통해 배달 로그 확인.

### 중복 배달

**원인**: 외부 서비스가 타임아웃으로 인해 재시도 중. shittim_chest가 LRU 캐시를 통해 자동으로 중복 감지.
**해결**: 유효한 재시도가 차단되는 경우 배달 ID 캐시 크기 증가. shittim_chest가 서비스의 타임아웃 기간 내에 응답하는지 확인 (GitHub: 10초).
