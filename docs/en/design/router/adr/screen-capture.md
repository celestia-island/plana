+++
title = "Screen Capture Architecture"
description = """Architecture decision record — Screen Capture Architecture."""
lang = "en"
category = "design"
subcategory = "router"
+++

# Screen Capture Architecture

- **Status**: Accepted
- **Date**: 2025-06-09
- **Authors**: Evernight Core Team

## Context

Evernight requires a cross-platform screen capture subsystem that feeds frames into the WebRTC streaming pipeline. The subsystem must work on Windows, macOS, and Linux with minimal latency and support both GPU-accelerated and software-based capture paths.

Key constraints:

- **Latency budget**: end-to-end capture-to-encode must stay under 16 ms at 60 FPS
- **Zero-copy where possible**: frames should reach the encoder without unnecessary copies
- **Hot-plug support**: displays and GPUs may appear or disappear at runtime
- **Permission handling**: macOS requires Screen Recording permission; Linux requires XDG/X11 or Wayland protocol access

## Decision

We adopt a **trait-based capture backend** with platform-specific implementations selected at compile time via Cargo features. Each backend implements the `FrameProvider` trait:

```rust
#[async_trait]
pub trait FrameProvider: Send + Sync {
    async fn enumerate_outputs(&self) -> Result<Vec<OutputInfo>>;
    async fn start_capture(&mut self, output: OutputId, config: CaptureConfig) -> Result<FrameReceiver>;
    async fn stop_capture(&mut self, output: OutputId) -> Result<()>;
}
```

### Backend selection

| Platform | Primary backend | Fallback |
|----------|----------------|----------|
| Windows  | DXGI Desktop Duplication | GDI BitBlt |
| macOS    | ScreenCaptureKit (SCStream) | CGWindowListCreateImage |
| Linux    | PipeWire (via xdg-desktop-portal) | XShm / XFixes |

### Frame lifecycle

1. `FrameProvider::start_capture` returns a `FrameReceiver` — a bounded MPSC channel carrying `Frame` structs
2. Each `Frame` owns a shared memory buffer (`Arc<FrameBuffer>`) that references GPU memory when available
3. The WebRTC encoder consumes from the channel; when all `Arc` references drop, the buffer is returned to a reuse pool
4. The capture thread never blocks on encoding — if the channel is full, the oldest frame is dropped and a `FrameDropped` counter is incremented

### Color space and format

- All backends negotiate the highest-bit-depth format available (BGRA8, NV12, or P010)
- A `ColorSpaceConverter` step handles transformation to the encoder's preferred format
- HDR metadata is preserved when the source provides it

## Consequences

### Positive

- Clean separation between capture and encoding allows independent testing
- Zero-copy path on Windows (DXGI) and macOS (ScreenCaptureKit) keeps latency well within budget
- Trait-based design permits third-party backends (e.g., virtual displays, test sources) without modifying core code
- Frame buffer pooling reduces allocation pressure under sustained capture

### Negative

- PipeWire on Linux introduces a D-Bus dependency that complicates headless/embedded scenarios
- macOS Screen Recording permission requires user interaction on first launch — no silent workaround
- Maintaining four backend implementations increases testing surface

### Risks and mitigations

- **Risk**: PipeWire API changes between distributions. **Mitigation**: pin to the stable `pw_stream` C API and vendor the Rust bindings.
- **Risk**: DXGI stuttering on hybrid GPU laptops. **Mitigation**: detect GPU topology at startup and prefer the integrated GPU for capture when the discrete GPU is rendering.

## References

- [DXGI Desktop Duplication API](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)
- [ScreenCaptureKit documentation](https://developer.apple.com/documentation/screencapturekit)
- [PipeWire screen cast portal](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org-freedesktop-portal-ScreenCast)
