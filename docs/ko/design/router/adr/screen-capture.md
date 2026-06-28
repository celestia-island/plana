# 화면 캡처 아키텍처

- **상태**: Accepted
- **날짜**: 2025-06-09
- **작성자**: Evernight Core Team

## 배경

Evernight는 WebRTC 스트리밍 파이프라인에 프레임을 공급하는 크로스 플랫폼 화면 캡처 서브시스템이 필요하다. 이 서브시스템은 Windows, macOS, Linux에서 최소 지연 시간으로 작동해야 하며 GPU 가속 및 소프트웨어 기반 캡처 경로를 모두 지원해야 한다.

주요 제약 사항:

- **지연 시간 예산**: 60 FPS 기준으로 종단 간 캡처-인코딩이 16ms 미만이어야 함
- **제로 카피 지향**: 프레임이 불필요한 복사 없이 인코더에 도달해야 함
- **핫플러그 지원**: 디스플레이와 GPU가 런타임에 나타나거나 사라질 수 있음
- **권한 처리**: macOS는 화면 녹화 권한이 필요하며, Linux는 XDG/X11 또는 Wayland 프로토콜 접근이 필요

## 결정

Cargo feature를 통해 컴파일 시점에 플랫폼별 구현이 선택되는 **트레이트 기반 캡처 백엔드**를 채택한다. 각 백엔드는 `FrameProvider` 트레이트를 구현한다:

```rust
#[async_trait]
pub trait FrameProvider: Send + Sync {
    async fn enumerate_outputs(&self) -> Result<Vec<OutputInfo>>;
    async fn start_capture(&mut self, output: OutputId, config: CaptureConfig) -> Result<FrameReceiver>;
    async fn stop_capture(&mut self, output: OutputId) -> Result<()>;
}
```

### 백엔드 선택

| 플랫폼   | 기본 백엔드                    | 폴백                      |
|----------|-------------------------------|--------------------------|
| Windows  | DXGI Desktop Duplication      | GDI BitBlt               |
| macOS    | ScreenCaptureKit (SCStream)   | CGWindowListCreateImage  |
| Linux    | PipeWire (xdg-desktop-portal) | XShm / XFixes            |

### 프레임 생명 주기

1. `FrameProvider::start_capture`는 `Frame` 구조체를 전달하는 제한된 MPSC 채널인 `FrameReceiver`를 반환한다
2. 각 `Frame`은 GPU 메모리를 참조하는(가능한 경우) 공유 메모리 버퍼(`Arc<FrameBuffer>`)를 소유한다
3. WebRTC 인코더가 채널에서 소비하며, 모든 `Arc` 참조가 해제되면 버퍼는 재사용 풀로 반환된다
4. 캡처 스레드는 인코딩에 의해 차단되지 않는다 — 채널이 가득 차면 가장 오래된 프레임이 폐기되고 `FrameDropped` 카운터가 증가한다

### 색상 공간 및 포맷

- 모든 백엔드는 사용 가능한 최고 비트 심도 포맷(BGRA8, NV12, 또는 P010)을 협상한다
- `ColorSpaceConverter` 단계가 인코더가 선호하는 포맷으로의 변환을 처리한다
- HDR 메타데이터는 소스가 제공할 때 보존된다

## 결과

### 긍정적 측면

- 캡처와 인코딩의 깔끔한 분리로 독립적인 테스트 가능
- Windows(DXGI)와 macOS(ScreenCaptureKit)의 제로 카피 경로로 지연 시간이 예산 내에 충분히 유지됨
- 트레이트 기반 설계로 핵심 코드를 수정하지 않고도 서드파티 백엔드(예: 가상 디스플레이, 테스트 소스) 허용
- 프레임 버퍼 풀링으로 지속적 캡처 시 할당 부담 감소

### 부정적 측면

- Linux의 PipeWire는 헤드리스/임베디드 시나리오를 복잡하게 만드는 D-Bus 의존성을 도입함
- macOS 화면 녹화 권한은 첫 실행 시 사용자 상호작용이 필요 — 무음 해결 방법 없음
- 네 가지 백엔드 구현 유지로 테스트 표면 증가

### 위험 요소와 완화 방안

- **위험**: 배포판 간 PipeWire API 변경. **완화**: 안정적인 `pw_stream` C API에 고정하고 Rust 바인딩을 벤더링한다.
- **위험**: 하이브리드 GPU 노트북에서 DXGI 끊김 현상. **완화**: 시작 시 GPU 토폴로지를 감지하고, 개별 GPU가 렌더링 중일 때는 통합 GPU를 캡처에 우선 사용한다.

## 참고 자료

- [DXGI Desktop Duplication API](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)
- [ScreenCaptureKit documentation](https://developer.apple.com/documentation/screencapturekit)
- [PipeWire screen cast portal](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org-freedesktop-portal-ScreenCast)
