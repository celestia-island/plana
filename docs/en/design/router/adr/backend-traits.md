# TerminalBackend / ViewportBackend / FileBackend Trait Abstractions

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight needs polymorphic backend interfaces so frontends (CLI, TUI, GUI) can consume any protocol uniformly. Without shared traits, each frontend would need protocol-specific code paths for terminal, graphical display, and file operations.

## Decision

Define three object-safe async traits in the crate root (always available, no feature flag):

- **`TerminalBackend`** — `read` / `write` / `resize` / `close`
- **`ViewportBackend`** — `render` / `input` / `clipboard` / `close`
- **`FileBackend`** — `list` / `stat` / `get` / `put` / `rm` / `mkdir` / `rename`

Each protocol backend implements the relevant traits. Frontends consume `Box<dyn TerminalBackend>` etc.

## Consequences

### Positive

- Frontends are protocol-agnostic; new backends (e.g., RDP) slot in without frontend changes.

### Negative

- Async trait objects require `Box::pin` overhead.
- Trait design must be stable since changing it breaks all implementors.
