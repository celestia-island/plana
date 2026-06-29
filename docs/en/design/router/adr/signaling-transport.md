+++
title = "Signaling Transport — Dual Unix Socket / TCP"
description = """Architecture decision record — Signaling Transport — Dual Unix Socket / TCP."""
lang = "en"
category = "design"
subcategory = "router"
+++

# Signaling Transport — Dual Unix Socket / TCP

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

The signaling client connects to a relay server to exchange WebRTC SDP offers/answers and ICE candidates. Originally it used only Unix domain sockets (`tokio::net::UnixStream`), which are unavailable on Windows. For cross-platform support, a fallback is needed.

## Decision

Implement a dual-transport `SignalingStream` enum that tries Unix domain socket first (on platforms that support it, detected by path format — starts with `/` or ends with `.sock`), falling back to TCP. The `TransportWriter` adapter abstracts over `AsyncWrite` for both transports. On non-Unix platforms, only TCP is available.

## Consequences

### Positive

- Cross-platform signaling; same JSON-RPC protocol over both transports

### Negative

- TCP signaling is unencrypted by default
- Windows users must ensure loopback-only binding
