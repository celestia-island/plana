+++
title = "RBAC 시스템 상세 설계 문서"
description = """Shittim Chest를 위한 완전한 역할 기반 접근 제어 시스템을 구현하며, 다음을 지원한다:"""
lang = "ko"
category = "design"
subcategory = "webui"
+++

# RBAC 시스템 상세 설계 문서

## 1. 목표

Shittim Chest를 위한 완전한 역할 기반 접근 제어 시스템을 구현하며, 다음을 지원한다:

- **사용자 관리**: 관리자가 사용자를 초대/생성/비활성화/삭제할 수 있음
- **그룹 관리**: 계정 그룹 지원, 사용자는 여러 그룹에 속할 수 있음
- **세분화된 권한**: 사용자가 특정 모델 Provider, MCP 도구, Layer3 Agent, IM 채널 등을 추가/수정/사용할 수 있는지 제어
- **기능 스위치**: 사용자가 자동 순항 모드 등 고급 기능을 사용할 수 있는지 제어
- **유연한 권한 부여 모드**: 관리자가 전역 통일 설정, 각 계정별 개별 설정 또는 계정 그룹 공유 중 선택 가능

## 2. 핵심 개념

### 2.1 역할 (Role)

| 역할 | 설명 |
| --- | --- |
| `admin` | 슈퍼 관리자, 모든 권한 보유, RBAC 자체 관리 가능 |
| `operator` | 운영자, 대부분의 리소스 관리 가능 (Provider, 채널, Agent 등) |
| `member` | 일반 멤버, 승인된 리소스 사용 가능 |
| `viewer` | 읽기 전용 사용자, 조회만 가능, 수정 불가 |

역할은 **사전 정의된** 것이며, 사용자 정의 역할은 제공하지 않는다 (구현 단순화). 각 사용자는 하나의 주 역할을 가질 수 있다.

### 2.2 권한 (Permission)

권한 형식: `<resource>.<action>`

| 카테고리 | 권한 | 설명 |
| --- | --- | --- |
| **Provider** | `provider.list` | Provider 목록 보기 |
| | `provider.create` | Provider 추가 |
| | `provider.update` | Provider 설정 수정 |
| | `provider.delete` | Provider 삭제 |
| | `provider.use` | Provider의 모델을 사용하여 대화 |
| **MCP 도구** | `mcp.list` | MCP 도구 목록 보기 |
| | `mcp.create` | MCP 도구 등록 |
| | `mcp.update` | MCP 도구 설정 수정 |
| | `mcp.delete` | MCP 도구 삭제 |
| | `mcp.use` | 대화에서 MCP 도구 사용 |
| **Agent** | `agent.list` | Agent 목록 보기 |
| | `agent.create` | Agent 생성 |
| | `agent.update` | Agent 설정 수정 |
| | `agent.delete` | Agent 삭제 |
| | `agent.use` | 분석 모드에서 Agent 사용 |
| **IM 채널** | `channel.list` | IM 채널 목록 보기 |
| | `channel.create` | IM 채널 생성 |
| | `channel.update` | 채널 설정 수정 |
| | `channel.delete` | 채널 삭제 |
| | `channel.use` | 채널을 통해 메시지 송수신 |
| **순항 모드** | `yolo.use` | 자동 순항 모드 사용 |
| **워크스페이스** | `workspace.list` | 워크스페이스 보기 |
| | `workspace.create` | 워크스페이스 생성 |
| | `workspace.manage` | 워크스페이스 관리 (삭제, 내보내기) |
| **장치** | `device.list` | 원격 장치 보기 |
| | `device.connect` | 원격 장치 연결 |
| **시스템** | `system.read` | 시스템 설정 보기 |
| | `system.write` | 시스템 설정 수정 |
| | `rbac.manage` | RBAC 관리 (사용자/그룹/권한) |
| **OAuth** | `oauth.read` | OAuth 설정 보기 |
| | `oauth.write` | OAuth 설정 수정 |

### 2.3 역할 기본 권한

| 권한 | admin | operator | member | viewer |
| --- | --- | --- | --- | --- |
| `provider.*` | ✅ | ✅ | `list` + `use` | `list` |
| `mcp.*` | ✅ | ✅ | `list` + `use` | `list` |
| `agent.*` | ✅ | ✅ | `list` + `use` | `list` |
| `channel.*` | ✅ | ✅ | `list` + `use` | `list` |
| `yolo.use` | ✅ | ✅ | ❌ (기본 비활성화) | ❌ |
| `workspace.*` | ✅ | ✅ | `list` + `create` | `list` |
| `device.*` | ✅ | ✅ | `list` + `connect` | `list` |
| `system.*` | ✅ | ❌ | ❌ | ❌ |
| `rbac.manage` | ✅ | ❌ | ❌ | ❌ |
| `oauth.*` | ✅ | ✅ | ❌ | ❌ |

### 2.4 권한 부여 모드

Provider, MCP, Agent, 채널 등 리소스에 대해 세 가지 권한 부여 모드를 지원한다:

| 모드 | 설명 | 적용 시나리오 |
| --- | --- | --- |
| **전역 설정** | 모든 사용자가 동일한 권한 공유 | 소규모 팀, 개인 사용 |
| **사용자별 설정** | 각 사용자가 독립적인 리소스 권한 보유 | 세밀한 제어가 필요한 시나리오 |
| **그룹별 설정** | 동일 그룹의 사용자가 권한 공유 | 부서/팀별 구분 |

관리자는 '권한 매트릭스' 페이지에서 권한 부여 모드를 선택한 후, 구체적인 허용/거부 규칙을 설정한다.

**우선순위**: 사용자별 설정 > 그룹별 설정 > 전역 설정 > 역할 기본 권한

## 3. 데이터베이스 스키마

### 3.1 신규 테이블

#### `rbac_groups` — 사용자 그룹

```sql
CREATE TABLE rbac_groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `rbac_user_groups` — 사용자-그룹 연결

```sql
CREATE TABLE rbac_user_groups (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id   UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);
```

#### `rbac_grants` — 권한 부여 (통합 테이블)

```sql
CREATE TABLE rbac_grants (
    id           UUID PRIMARY KEY,
    -- 권한 부여 대상 (삼택일)
    scope        VARCHAR(16) NOT NULL, -- 'global' | 'group' | 'user'
    user_id      UUID REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id     UUID REFERENCES rbac_groups(id) ON DELETE CASCADE,
    -- 권한
    permission   VARCHAR(64) NOT NULL, -- 예: 'provider.use', 'yolo.use'
    resource_id  VARCHAR(128),         -- 선택: 특정 리소스로 제한 (provider name, channel id 등), NULL은 해당 카테고리 전체 리소스
    -- 권한 부여 유형
    granted      BOOLEAN NOT NULL DEFAULT TRUE, -- TRUE=허용, FALSE=거부
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 제약: scope와 해당 FK가 일치해야 함
    CONSTRAINT rbac_grants_scope_check CHECK (
        (scope = 'global' AND user_id IS NULL AND group_id IS NULL) OR
        (scope = 'user'   AND user_id IS NOT NULL AND group_id IS NULL) OR
        (scope = 'group'  AND user_id IS NULL AND group_id IS NOT NULL)
    )
);
CREATE INDEX idx_rbac_grants_user ON rbac_grants(user_id);
CREATE INDEX idx_rbac_grants_group ON rbac_grants(group_id);
CREATE INDEX idx_rbac_grants_permission ON rbac_grants(permission);
```

### 3.2 기존 테이블 수정

#### `auth_users`에 역할 필드 추가

```sql
ALTER TABLE auth_users ADD COLUMN role VARCHAR(16) NOT NULL DEFAULT 'member';
-- 마이그레이션: is_admin=true인 사용자를 'admin'으로 설정
UPDATE auth_users SET role = 'admin' WHERE is_admin = TRUE;
```

`is_admin` 필드는 호환성을 위해 유지하지만, 새 코드는 `role`을 우선 사용한다.

### 3.3 권한 확인 로직 (의사 코드)

```rust
fn has_permission(user, permission, resource_id=None) -> bool {
    // 1. admin 역할은 바로 통과
    if user.role == "admin" { return true; }

    // 2. 일치하는 모든 grants 수집, 우선순위에 따라 정렬
    let grants = [];

    // 2a. 역할 기본 권한 (가장 낮은 우선순위)
    grants.push(role_defaults(user.role, permission));

    // 2b. 전역 설정
    grants.extend(query_grants(scope="global", permission, resource_id));

    // 2c. 그룹 설정 (사용자가 속한 모든 그룹)
    for group in user.groups:
        grants.extend(query_grants(scope="group", group_id=group.id, permission, resource_id));

    // 2d. 사용자 레벨 설정 (가장 높은 우선순위)
    grants.extend(query_grants(scope="user", user_id=user.id, permission, resource_id));

    // 3. 우선순위: user > group > global > role_default
    // 동일 scope 내에서 denied가 granted보다 우선
    // user scope에 denied가 있으면 → 거부
    // group scope에 denied가 있으면 → 거부 (user scope에 granted가 있는 경우 제외)
    // 최종 결과
    resolve_grants(grants)
}
```

## 4. API 설계

### 4.1 사용자 관리 (`/api/rbac/users`)

| 메서드 | 경로 | 권한 | 설명 |
| --- | --- | --- | --- |
| GET | `/api/rbac/users` | `rbac.manage` | 모든 사용자 목록 (역할, 그룹 포함) |
| POST | `/api/rbac/users` | `rbac.manage` | 사용자 초대 (이메일 발송 또는 계정 생성) |
| PUT | `/api/rbac/users/:id` | `rbac.manage` | 사용자 역할 업데이트, 활성화/비활성화 |
| DELETE | `/api/rbac/users/:id` | `rbac.manage` | 사용자 삭제 |

### 4.2 그룹 관리 (`/api/rbac/groups`)

| 메서드 | 경로 | 권한 | 설명 |
| --- | --- | --- | --- |
| GET | `/api/rbac/groups` | `rbac.manage` | 모든 그룹 목록 |
| POST | `/api/rbac/groups` | `rbac.manage` | 그룹 생성 |
| PUT | `/api/rbac/groups/:id` | `rbac.manage` | 그룹 업데이트 (이름, 설명) |
| DELETE | `/api/rbac/groups/:id` | `rbac.manage` | 그룹 삭제 |
| POST | `/api/rbac/groups/:id/members` | `rbac.manage` | 멤버 추가 |
| DELETE | `/api/rbac/groups/:id/members/:userId` | `rbac.manage` | 멤버 제거 |

### 4.3 권한 관리 (`/api/rbac/grants`)

| 메서드 | 경로 | 권한 | 설명 |
| --- | --- | --- | --- |
| GET | `/api/rbac/grants` | `rbac.manage` | 모든 권한 규칙 목록 (?scope=&permission= 필터 지원) |
| PUT | `/api/rbac/grants` | `rbac.manage` | 권한 일괄 설정 (전체 규칙 목록 전달, 해당 scope의 규칙 덮어쓰기) |
| DELETE | `/api/rbac/grants/:id` | `rbac.manage` | 단일 규칙 삭제 |

### 4.4 권한 확인 (`/api/rbac/check`)

| 메서드 | 경로 | 권한 | 설명 |
| --- | --- | --- | --- |
| GET | `/api/rbac/check?permission=xxx&resource_id=yyy` | (모든 인증 사용자) | 현재 사용자에게 지정된 권한이 있는지 확인 |
| GET | `/api/rbac/my-permissions` | (모든 인증 사용자) | 현재 사용자의 모든 유효 권한 목록 반환 |

### 4.5 리소스 가시성 개선

기존 리소스 API에 권한 필터링 추가 필요:

- `GET /api/chat/providers` → 현재 사용자에게 `provider.list` 권한이 있는 Provider만 반환, `provider.use` 권한이 있는 모델만 표시
- `GET /api/channel` → `channel.list` 권한이 있는 채널만 반환
- 순항 모드 시작 전 → `yolo.use` 권한 확인

## 5. 프론트엔드 설계 (Plana)

### 5.1 RbacView 재구성

세 개의 탭으로 구분:

#### 탭 1: 사용자 관리

- 사용자 목록 테이블: 아바타, 사용자명, 이메일, 역할 (드롭다운 전환), 그룹 태그, 상태 (활성/비활성), 작업
- 사용자 초대 버튼 → 모달 팝업 (사용자명/이메일/비밀번호 입력, 역할 선택)
- 행 작업: 역할 편집, 비활성화/활성화, 삭제

#### 탭 2: 그룹 관리

- 그룹 목록 테이블: 이름, 설명, 멤버 수, 작업
- 그룹 생성 → 모달 팝업
- 그룹 클릭 → 멤버 목록 확장, 멤버 추가/제거 가능

#### 탭 3: 권한 매트릭스

- 좌측 상단에서 권한 부여 모드 선택: 전역 / 그룹별 / 사용자별
- 그룹 또는 사용자 선택 후 권한 매트릭스 테이블 표시:
  - 행: 리소스 카테고리 (Provider, MCP, Agent, 채널, 순항 모드...)
  - 열: 작업 (목록, 생성, 수정, 삭제, 사용)
  - 셀: 3상태 전환 (✅ 허용 / ❌ 거부 / ➖ 기본값 상속)
- 특정 리소스 ID에 대한 세밀한 제어 (예: 특정 Provider만 사용 허용)

### 5.2 내비게이션 권한 제어

- 사이드바 항목을 현재 사용자 권한에 따라 동적으로 표시/숨김
- 라우트 가드에 권한 확인 추가, 권한 없을 시 403 페이지로 리디렉션
- 작업 버튼 (예: "Provider 추가")을 권한에 따라 표시/숨김

## 6. 구현 단계

### Phase 1: 백엔드 기초

1. 신규 데이터베이스 마이그레이션 추가 (`rbac_groups`, `rbac_user_groups`, `rbac_grants` 테이블 + auth_users.role 필드)
1. 신규 SeaORM 엔티티 모델 추가
1. RBAC API 라우트 구현 (users, groups, grants CRUD)
1. 권한 확인 미들웨어/extractor 구현
1. JWT claims에 role 필드 추가

### Phase 2: 백엔드 통합

1. 기존 리소스 API (providers, channels 등)에 권한 확인 추가
1. `/api/rbac/check` 및 `/api/rbac/my-permissions` 구현
1. arona의 리소스 요청을 권한 필터링에 맞게 수정

### Phase 3: 프론트엔드 UI

1. arona의 RbacView 재구성 (사용자/그룹/권한 매트릭스 세 개 탭)
1. 사이드바 및 라우트의 권한 가드 구현
1. arona에서 권한에 따라 기능 숨김/비활성화 (예: 순항 모드 버튼)

## 7. 보안 고려사항

- `admin` 역할의 권한은 rbac_grants로 덮어쓸 수 없음 (하드코딩 통과)
- 권한 확인은 미들웨어 레이어에서 통합 실행, 비즈니스 코드의 수동 확인에 의존하지 않음
- 민감한 작업 (사용자 삭제, 권한 수정)은 감사 로그 기록
- JWT에는 role만 포함, 구체적인 권한은 매번 DB에서 실시간 조회 (권한 변경 후 토큰 미갱신 방지)
