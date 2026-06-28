+++
title = "내장형 프론트엔드 전략"
description = """shittim-chest는 두 가지 프론트엔드 호스팅 모드를 지원한다: Dev 모드에서는 `dev.py`가 프론트엔드 소스를 감시하고 변경 시 `pnpm build`를 트리거하며, 백엔드가 `:3000`에서 정적 파일과 API를 모두 제"""
lang = "ko"
category = "design"
subcategory = "webui"
+++

# 내장형 프론트엔드 전략

## 개요

shittim-chest는 두 가지 프론트엔드 호스팅 모드를 지원한다: Dev 모드에서는 `dev.py`가 프론트엔드 소스를 감시하고 변경 시 `pnpm build`를 트리거하며, 백엔드가 `:3000`에서 정적 파일과 API를 모두 제공한다. Release 모드에서는 프론트엔드 정적 파일이 컴파일 시점에 Rust 바이너리에 내장되어 `:80`에서 제공된다. 모드는 `embedded-frontend` Cargo 기능을 통해 전환되며, `#[cfg(feature = "embedded-frontend")]`를 사용한 코드 레벨 조건부 컴파일이 적용된다.

## 아키텍처 비교

```mermaid
flowchart TB
    subgraph Dev[Dev 모드: dev.py + 백엔드]
        D1[dev.py가 프론트엔드 src 감시] --> D2[pnpm build → dist/]
        D2 --> D3[shittim_chest :3000 정적 + API 제공]
    end
    subgraph Release[Release 모드: 내장형]
        R1[브라우저] --> R2[shittim_chest :80]
        R2 --> R3[API + LLM]
        R2 --> R4[/static/*\n내장 SPA]
    end
```

| 차원 | Dev (기능 없음) | Release (embedded-frontend) |
| --- | --- | --- |
| 프론트엔드 소스 | Vite로 빌드, 백엔드가 제공 | `include_dir!` 컴파일 시점 내장 |
| 핫 리로드 | dev.py를 통한 자동 재빌드 | 지원 안 함 (정적) |
| API 요청 라우팅 | 브라우저 직접 연결 (동일 출처) | 브라우저 직접 연결 |
| 바이너리 크기 | 백엔드만 | + 프론트엔드 dist/ 디렉터리 |
| Node 필요 | 예 (빌드 전용) | 아니요 |
| 시작 방법 | `dev.py` (감시 + 재빌드) | `just up` 원샷 실행 |

## 구현 세부 사항

### 조건부 컴파일

```rust
# [cfg(feature = "embedded-frontend")]
static ARONA_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../dist/arona");

async fn serve_arona() -> impl IntoResponse {
    #[cfg(feature = "embedded-frontend")]
    {
        // 컴파일 시점 내장 Dir에서 읽기
    }
    #[cfg(not(feature = "embedded-frontend"))]
    {
        // 파일시스템 ./dist/arona/index.html에서 읽기
    }
}
```

조건부 컴파일은 모듈 레벨이 아닌 **함수 본문 레벨**에서 작동하여, 공개 API를 두 모드에서 동일하게 유지한다.

### SPA 폴백

애플리케이션은 단일 페이지 애플리케이션이다. 정적 에셋과 일치하지 않는 모든 경로는 `index.html`을 반환한다:

```text
GET /               → index.html
GET /chat/123       → index.html (프론트엔드 라우터가 처리)
GET /backend        → index.html
GET /backend/providers → index.html (프론트엔드 라우터가 처리)
```

### MIME 타입 감지

정적 파일 제공은 파일 확장자에 따라 올바른 Content-Type을 반환한다:

| 확장자 | Content-Type |
| --- | --- |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.html` | `text/html` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff/.woff2` | `font/woff2` |
| 기타 | `application/octet-stream` |

## Dockerfile 내 프론트엔드 빌드

```text
Stage 1 (프론트엔드):
  node:22-slim → pnpm install → pnpm build:all → /app/dist/arona/

Stage 2 (빌더):
  rust:1.85-slim → COPY /app/dist/ → cargo build --features embedded-frontend

Stage 3 (런타임):
  debian:bookworm-slim → COPY 바이너리 → ENTRYPOINT ["./shittim_chest"]
```

프론트엔드 빌드와 Rust 컴파일은 동일한 Dockerfile 내에서 완료된다. 최종 런타임 이미지에는 컴파일된 바이너리만 포함된다.

## 설계 결정

1. **Dev 모드는 자동 재빌드를 위해 dev.py를 사용한다**: `dev.py`가 프론트엔드 소스를 감시하고 변경 시 재빌드하며, 백엔드가 하나의 포트에서 모든 것을 제공한다.
1. **Release 모드는 리버스 프록시가 필요하지 않다**: 바이너리가 SPA를 내장하여 단일 프로세스 배포를 가능하게 하고 운영 복잡성을 줄인다.
1. **프론트엔드는 런타임에 동적으로 로드되지 않는다**: 파일시스템 의존성과 버전 불일치를 방지한다. Release 이미지에는 단일 바이너리 파일만 포함된다.
1. **단일 SPA**: 프론트엔드는 `/`에서 제공되며, 관리자 패널은 `/backend`에 위치한다.
