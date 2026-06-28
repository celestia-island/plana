+++
title = "CLI 로깅 규칙"
description = """shittim-chest CLI 래퍼의 로그 출력은 entelecheia와 일관된 규칙을 따르며, `tracing` 생태계를 사용하여 압축된 사람이 읽기 쉬운 형식으로 stderr에 출력한다."""
lang = "ko"
category = "design"
subcategory = "webui"
+++

# CLI 로깅 규칙

## 개요

shittim-chest CLI 래퍼의 로그 출력은 entelecheia와 일관된 규칙을 따르며, `tracing` 생태계를 사용하여 압축된 사람이 읽기 쉬운 형식으로 stderr에 출력한다.

## 프레임워크 선택

| 구성 요소 | 선택 | 이유 |
| --- | --- | --- |
| 로깅 프레임워크 | `tracing` | Rust 생태계 표준, entelecheia와 일관성 유지 |
| 구독자 | `tracing-subscriber` fmt 레이어 | 압축 출력, JSON 파싱 불필요 |
| 시간 형식 | `ShortTimer` (HH:MM:SS) | 터미널 친화적, entelecheia CLI와 일관성 |
| 출력 대상 | stderr | stdout과 분리, 파이프 간섭 없음 |

## 초기화 코드

```rust
use chrono::Local;
use tracing_subscriber::fmt::time::FormatTime;

struct ShortTimer;

impl FormatTime for ShortTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{} ", now.format("%H:%M:%S"))
    }
}

// 초기화
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(&args.log_level))
    .with_target(false)          // 모듈 경로 숨김
    .with_timer(ShortTimer)      // HH:MM:SS 형식
    .compact()                   // 압축 모드
    .with_writer(std::io::stderr) // stderr로 출력
    .init();
```

## 형식 비교

| 모드 | 예제 출력 | 사용 사례 |
| --- | --- | --- |
| CLI (현재) | `14:23:05  INFO 네트워크 shittim-chest 생성 중...` | 개발, 운영 |
| 서버 (향후) | `{"timestamp":"...","level":"INFO","message":"..."}` | 프로덕션 로그 수집 |

## --log-level 매개변수

CLI는 `--log-level` / `-l` 매개변수를 허용한다 (기본값 `info`):

```text
shittim-chest --log-level debug dev
shittim-chest -l trace status
```

지원 레벨: `trace`, `debug`, `info`, `warn`, `error`.

## 로그 레벨 사용 규칙

| 레벨 | 목적 | 일반적인 CLI 시나리오 |
| --- | --- | --- |
| `info` | 중요한 작업 | 컨테이너 생성/시작/정지, 마이그레이션 시작/완료 |
| `warn` | 잠재적 문제 | 마이그레이션 재시도, 컨테이너 존재하지만 비정상 상태 |
| `error` | 오류 | 컨테이너 충돌, 마이그레이션 실패, 네트워크 생성 실패 |
| `debug` | 디버그 정보 | (현재 미사용, 향후 예약) |
| `trace` | 상세 흐름 | (현재 미사용, 향후 예약) |

## 설계 원칙

1. **CLI는 오류를 삼키지 않는다**: 모든 오류는 `anyhow::Result`를 통해 상위로 전파되며, `main()`이 자동으로 오류 체인을 출력한다.
1. **모든 작업 시작에 로그가 있다**: `네트워크 생성 중...`, `마이그레이션 실행 중...`, `shittim_chest 빌드 중...` — 사용자는 CLI가 무엇을 하고 있는지 알 수 있다.
1. **모든 작업 완료에 확인이 있다**: `shittim-chest가 0.0.0.0:80에서 시작됨`, `모든 서비스가 시작됨`.
1. **조용히 성공하는 작업은 로깅되지 않는다**: `ensure_network`는 네트워크가 이미 존재하면 출력하지 않아 노이즈를 방지한다.
1. **컨테이너 로그는 Docker API를 통해 가져온다**: CLI 자체는 비즈니스 로그를 작성하지 않으며, 오케스트레이션 작업 로그만 기록한다.

## entelecheia와의 정렬

| 기능 | entelecheia CLI | shittim-chest CLI | 정렬됨 |
| --- | --- | --- | --- |
| 프레임워크 | `tracing` | `tracing` | ✅ |
| 시간 형식 | `ShortTimer` (HH:MM:SS) | `ShortTimer` (HH:MM:SS) | ✅ |
| 출력 대상 | stderr | stderr | ✅ |
| 압축 모드 | `.compact()` | `.compact()` | ✅ |
| 대상 숨김 | `.with_target(false)` | `.with_target(false)` | ✅ |
| --log-level | 지원됨 | 지원됨 | ✅ |

두 프로젝트의 CLI 로그 출력은 시각적으로 동일하여, 개발자가 두 프로젝트 간에 쉽게 전환할 수 있다.
