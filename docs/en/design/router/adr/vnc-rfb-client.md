# VNC (RFB) Protocol Client

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight needs a graphical remote desktop client supporting VNC. The RFB protocol (RFC 6143) is the standard for VNC. Options: use an existing Rust crate, bind to libvncclient, or implement from scratch.

## Decision

Implement a pure Rust RFB 003.008 client from scratch. Supports version handshake, security negotiation (None, VncAuth), DES challenge-response authentication, pixel format negotiation, and Raw/CopyRect encoding. Implements `ViewportBackend` trait for frontend integration.

## Consequences

### Positive

- No C dependency; full control over the protocol flow.

### Negative

- ZRLE and Tight encoding not yet implemented (heavy to implement).
- DES implementation needed for VNC auth.
