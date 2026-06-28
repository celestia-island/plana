# Error Handling — thiserror with crate Result

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight needs a unified error type across all modules (screen, SSH, hardware, networking, tunneling). The library must expose a clean error API while keeping internal error handling ergonomic.

## Decision

Use `thiserror` to derive `EvernightError` enum with per-variant `#[error(...)]` display implementations. Define `pub type Result<T> = std::result::Result<T, EvernightError>` as the crate-wide result type. Each variant captures domain-specific context (ScreenCapture, Ssh, Tunnel, etc.) as String rather than wrapping external error types, to avoid leaking internal dependency details.

## Consequences

### Positive

- Stable public error API; consumers match on enum variants without knowing internals

### Negative

- Some information loss from `String` conversion
- No structured error chaining
