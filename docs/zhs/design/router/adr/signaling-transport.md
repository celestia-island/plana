# 双 Unix Socket / TCP 信令传输

- **Status**: 已采纳
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

信令客户端连接到中继服务器，以交换 WebRTC SDP offer/answer 和 ICE candidate。最初仅使用 Unix domain socket（`tokio::net::UnixStream`），但 Windows 上不可用。为了实现跨平台支持，需要有一种回退方案。

## Decision

实现双传输 `SignalingStream` 枚举，先尝试 Unix domain socket（在支持该功能的平台上，通过路径格式检测——以 `/` 开头或以 `.sock` 结尾），失败则回退到 TCP。`TransportWriter` 适配器为两种传输统一抽象 `AsyncWrite`。在非 Unix 平台上，仅提供 TCP。

## Consequences

### Positive

- 跨平台信令；两种传输均使用相同的 JSON-RPC 协议

### Negative

- TCP 信令默认未加密
- Windows 用户必须确保仅绑定到 loopback
