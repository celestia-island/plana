+++
title = "Async Runtime — tokio"
description = """Architecture decision record — Async Runtime — tokio."""
lang = "en"
category = "design"
subcategory = "router"
+++

# Async Runtime — tokio

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight needs an async runtime for network I/O (SSH, WebRTC, TCP tunnels), timer-based operations (frame capture loops, protocol timeouts), and concurrent task management. The Rust ecosystem offers tokio, async-std, and smol.

## Decision

Use `tokio` as the sole async runtime. tokio is the de facto standard in the Rust async ecosystem with the richest driver support. Key dependencies (russh, webrtc, reqwest) already require tokio. Use `tokio::runtime::Handle::current()` for spawning from synchronous contexts.

## Consequences

### Positive

- Ecosystem compatibility; no runtime bridging needed

### Negative

- tokio is a heavy dependency
- Cannot use async-std or smol crates that lack tokio support
