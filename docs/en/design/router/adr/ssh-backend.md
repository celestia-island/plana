+++
title = "SSH Backend — russh with Shared Connection"
description = """Architecture decision record — SSH Backend — russh with Shared Connection."""
lang = "en"
category = "design"
subcategory = "router"
+++

# SSH Backend — russh with Shared Connection

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight needs SSH for remote shell, file operations, and terminal access. Initially the codebase had three separate SSH handler implementations (FileHandler, SshHandler, TerminalHandler) with duplicated connection logic. The project mandates pure Rust (no `Command::new("ssh")` shell-outs).

## Decision

Use `russh` (pure Rust SSH-2 implementation) as the SSH backend. Consolidate all SSH handler implementations into a single `DefaultSshHandler` in `remote/connection.rs` with a shared `connect_session()` function. All SSH operations (shell, files, terminal) share this connection abstraction.

## Consequences

### Positive

- Single source of truth for SSH auth logic
- Easy to add connection pooling later

### Negative

- `russh` may lag behind OpenSSH in edge cases
- No built-in SSH agent forwarding yet
