+++
title = "SSH Config Parser"
description = """Architecture decision record — SSH Config Parser."""
lang = "en"
category = "design"
subcategory = "router"
+++

# SSH Config Parser

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Users expect `evernight` to read `~/.ssh/config` for host aliases, jump hosts, and key files, just like the `ssh` command. Without this, users must repeat all connection parameters on every invocation.

## Decision

Implement a pure Rust SSH config parser that handles the standard directives: `Host`, `HostName`, `User`, `Port`, `IdentityFile`, `ProxyJump`, `ForwardAgent`, `ServerAliveInterval`, etc. Glob pattern matching for `Host` entries. Integrated with the connection pool for transparent config-based connections.

## Consequences

### Positive

- Drop-in SSH compatibility; jump host resolution from config.

### Negative

- Must track OpenSSH config syntax changes.
- `Match` directive support is complex.
